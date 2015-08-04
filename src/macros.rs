#[cfg(feature = "debug")]
macro_rules! debug {
    ($fmt:expr) => (println!($fmt));
    ($fmt:expr, $($arg:tt)*) => (println!($fmt, $($arg)*));
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($fmt:expr) => ();
    ($fmt:expr, $($arg:tt)*) => ();
}

#[cfg(feature = "output")]
macro_rules! output {
    ($fmt:expr) => (println!($fmt));
    ($fmt:expr, $($arg:tt)*) => (println!($fmt, $($arg)*));
}

#[cfg(not(feature = "output"))]
macro_rules! output {
    ($fmt:expr) => ();
    ($fmt:expr, $($arg:tt)*) => ();
}
