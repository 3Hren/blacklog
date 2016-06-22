use libc;

#[inline]
pub fn id() -> usize {
    __get_id()
}

#[cfg(unix)]
#[inline]
fn __get_id() -> usize {
    unsafe {
        libc::pthread_self() as usize
    }
}

#[cfg(not(all(unix)))]
#[inline]
fn __get_id() -> usize {
    0
}

#[cfg(test)]
mod tests {
    use super::{id};

    #[test]
    fn test_id() {
        assert!(id() > 0);
    }

    #[cfg(feature="benchmark")]
    use test::{self, Bencher};

    #[cfg(feature="benchmark")]
    #[bench]
    fn bench_id(b: &mut Bencher) {
        b.iter(|| {
            let id = id();
            test::black_box(id);
        });
    }

    #[cfg(feature="benchmark")]
    #[bench]
    fn name(b: &mut Bencher) {
        b.iter(|| {
            test::black_box(::std::thread::current().name());
        });
    }
}
