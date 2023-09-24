

#[macro_export]
macro_rules! timeit {
    ($format_str:expr, $code:expr) => {
        {
            let start = Utc::now();
            let out = $code;
            writeln!(
                $format_str,
                (Utc::now() - start).num_milliseconds()
            );
            out
        }
    };
}

/* usage
timeit(
  "the thing took {} ms",
  {
    # the thing that takes time
  }
)
 */