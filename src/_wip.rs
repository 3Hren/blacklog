trait Mutant : Send + Sync {
    fn mutate(&self, rec: &mut Record, f: &Fn(&mut Record));
}

struct FalloutMutant;

impl FalloutMutant {
    fn mutate(&self, rec: &mut Record, f: &Fn(&mut Record)) {
        let v = 42;
        let m = &[Meta::new("a1", &v)];
        // let meta = MetaLink::next(m, Some(rec.meta));
        // let mut rec2 = *rec;
        // rec2.meta = &meta;

        // f(&mut rec2)
        f(rec)
    }
}

impl SomeHandler {
    fn handle_<'a>(&self, rec: &mut Record<'a>, mutants: &[Box<Mutant>]) {
        println!("{:?}", mutants.len());

        match mutants.iter().next() {
            Some(mutant) => {
                mutant.mutate(rec, &|rec| {
                    self.handle_(rec, &mutants[1..])
                })
            }
            None => {
                let mut wr: Vec<u8> = Vec::new();
                // self.layout.format(rec, &mut wr);
            }
        }
    }
}

struct Scope<'a, F: FnOnce() -> &'static str> {
    logger: &'a Logger,
    f: F,
}

impl<'a, F: FnOnce() -> &'static str> Drop for Scope<'a, F> {
    fn drop(&mut self) {
        let l = &self.logger;
        // log!(l, 42, "fuck you");
    }
}
