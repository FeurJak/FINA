#[macro_export]
macro_rules! ct_for {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! ct_rev_for {
    (($i:ident in $end:tt.. $start:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::rev_cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! ct_for_unroll2 {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! ct_for_unroll4 {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! ct_for_unroll6 {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! ct_rev_for_unroll6 {
    (($i:ident in $end:tt.. $start:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
            $crate::rev_cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! ct_for_unroll8 {
    (($i:ident in $start:tt.. $end:tt) $code:expr) => {{
        let mut $i = $start;
        loop {
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
            $crate::cycle!($i, $end, $code);
        }
    }};
}

#[macro_export]
macro_rules! cycle {
    ($i:ident, $end:tt, $code:expr) => {{
        if $i < $end {
            $code
        } else {
            break;
        }
        $i += 1;
    }};
}

#[macro_export]
macro_rules! rev_cycle {
    ($i:ident, $end:tt, $code:expr) => {{
        if $end < $i {
            $i -= 1;
            $code
        } else {
            break;
        }
    }};
}

#[macro_export]
macro_rules! assert_all_eq_len {
    ([$head:expr, $($tail:expr),+ $(,)?]) => {{
        $(
            assert_eq!(
                $head.len(),
                $tail.len(),
            );
        )+
    }};
    ([$head:expr, $($tail:expr),+], $($arg:tt)+) => {{
        let __format = core::format_args!($($arg)+);
        $(
            assert_eq!(
                $head.len(),
                $tail.len(),
                "{}", __format,
            );
        )+
    }};
}

#[macro_export]
macro_rules! from_variant {
    ($to:ty, $kind:ident, $from:ty) => {
        impl From<$from> for $to {
            #[inline]
            fn from(t: $from) -> Self {
                Self::$kind(t)
            }
        }
    };
}

#[macro_export]
macro_rules! cfg_into_iter {
    ($e:expr) => {{ $e.into_iter() }};
    ($e:expr, $min_len:expr) => {{ $e.into_iter() }};
}

#[macro_export]
macro_rules! cfg_iter {
    ($e:expr) => {{ $e.iter() }};
    ($e:expr, $min_len:expr) => {{ $e.iter() }};
}

#[macro_export]
macro_rules! cfg_iter_mut {
    ($e:expr) => {{ $e.iter_mut() }};
    ($e:expr, $min_len:expr) => {{ $e.iter_mut() }};
}

#[macro_export]
macro_rules! cfg_chunks {
    ($e:expr, $size:expr) => {{
        $e.chunks($size)
    }};
}

#[macro_export]
macro_rules! cfg_chunks_mut {
    ($e:expr, $size:expr) => {{
        $e.chunks_mut($size)
    }};
}

#[macro_export]
macro_rules! cfg_reduce {
    ($e:expr, $default:expr, $op:expr) => {{ $e.fold($default(), $op) }};
}
