// Helper: run a blocking job on a worker thread and deliver the result to a
// main-thread callback through the GTK main loop (port of the QThread pattern).
// glib's invoke APIs require Send closures, so results are marshalled through a
// channel polled on the main loop.
use gtk4::glib;

pub fn spawn_async<R, W, D>(work: W, done: D)
where
    R: Send + 'static,
    W: FnOnce() -> R + Send + 'static,
    D: FnOnce(R) + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel::<R>();
    std::thread::spawn(move || {
        let _ = tx.send(work());
    });
    let mut done = Some(done);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(result) => {
                if let Some(d) = done.take() {
                    d(result);
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
        }
    });
}

// Helper: worker thread that streams intermediate messages (port of a QThread
// emitting signals per line) plus a final result, both delivered on the main loop.
pub fn spawn_stream<T, R, W, I, F>(work: W, on_item: I, on_finished: F)
where
    T: Send + 'static,
    R: Send + 'static,
    W: FnOnce(std::sync::mpsc::Sender<T>) -> R + Send + 'static,
    I: Fn(&T) + 'static,
    F: Fn(&R) + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel::<T>();
    let (rtx, rrx) = std::sync::mpsc::channel::<R>();
    std::thread::spawn(move || {
        let result = work(tx);
        let _ = rtx.send(result);
    });
    let mut on_finished = Some(on_finished);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        while let Ok(item) = rx.try_recv() {
            on_item(&item);
        }
        match rrx.try_recv() {
            Ok(result) => {
                if let Some(f) = on_finished.take() {
                    f(&result);
                }
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
        }
    });
}

/// Create a secondary window (dialog / tool window) correctly parented to the
/// running application.
///
/// A bare `gtk4::Window::new()` is not attached to the `Application` and has no
/// transient parent, so the window manager may render it without decorations and
/// the close button then does nothing. Python passes `parent=None` for most of
/// these dialogs, but Qt still attaches them to the application, so parenting
/// them here matches the Python behaviour while restoring the close button.
pub fn sub_window(app_title: &str) -> gtk4::Window {
    use gtk4::prelude::*;
    let win = gtk4::Window::new();
    win.set_title(Some(app_title));
    let app = gtk4::Application::default();
    win.set_application(Some(&app));
    if let Some(parent) = app.active_window() {
        win.set_transient_for(Some(&parent));
    }
    win
}
