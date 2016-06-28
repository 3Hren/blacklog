// #![feature(test)]
//
// extern crate blacklog;
// extern crate test;
//
// use test::Bencher;
//
// use blacklog::{Meta, MetaList, Record};
//
// /// This benchmark demonstrates, that creating an inactive record is very cheap.
// #[bench]
// fn new(b: &mut Bencher) {
//     b.iter(|| {
//         Record::new(0, line!(), module_path!(), format_args!("le message"), &MetaList::new(&[]));
//     });
// }
//
// /// This benchmark demonstrates, that creating an inactive record is very cheap, even with comlex
// /// formatting pattern and meta attributes.
// #[bench]
// fn new_with_format_and_meta6(b: &mut Bencher) {
//     b.iter(|| {
//         Record::new(0, line!(), module_path!(),
//             format_args!("le message: {}, {}", 42, "maybe later"),
//             &MetaList::new(&[Meta::new("meta#1", &42),
//             Meta::new("meta#1", &42),
//             Meta::new("meta#1", &42),
//             Meta::new("meta#1", &42),
//             Meta::new("meta#1", &42),
//             Meta::new("meta#1", &42)]));
//     });
// }
//
// #[bench]
// fn activate_with_format(b: &mut Bencher) {
//     b.iter(|| {
//         Record::new(0, line!(), module_path!(), format_args!("le {}", "message"), &MetaList::new(&[]))
//             .activate();
//     });
// }
//
// #[bench]
// fn activate_without_format(b: &mut Bencher) {
//     b.iter(|| {
//         Record::new(0, line!(), module_path!(), format_args!("le message"), &MetaList::new(&[]))
//             .activate();
//     });
// }
