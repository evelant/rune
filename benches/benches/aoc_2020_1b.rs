//! Benchmark of tgolsson's AoC 2020 solutions.
//!
//! Source: https://github.com/tgolsson/aoc-2020

#![feature(test)]

extern crate test;

use test::Bencher;

const INPUT: &str = include_str!("data/aoc_2020_1.txt");

#[bench]
fn aoc_2020_1b(b: &mut Bencher) -> rune::Result<()> {
    let mut data = rune::runtime::Vec::new();

    for line in INPUT.split('\n').filter(|s| !s.is_empty()) {
        data.push_value(str::parse::<i64>(line)?)?;
    }

    let mut vm = rune_tests::rune_vm! {
        mod iter {
            pub fn all_pairs(data) {
               let count = data.len();

               for i in 0..count {
                   let a = data[i];
                   for j in (i+1)..count {
                        yield [a, data[j]]
                   }
               }
            }

            pub fn all_triples(data) {
               let count = data.len();

               for i in 0..count {
                   let a = data[i];
                   for j in (i + 1)..count {
                       let b = data[j];
                       for k in (j+1)..count {
                          yield [a, b, data[k]]
                       }
                   }
               }
            }
        }

        fn filter_inner(items) {
            while let Some(i) = items.next() {
                if i.iter().sum() == 2020 {
                    return i.iter().product();
                }
            }
        }

        pub fn main(lines) {
            lines.sort::<int>();
            (filter_inner(iter::all_pairs(lines)), filter_inner(iter::all_triples(lines)))
        }
    };

    let entry = rune::Hash::type_hash(&["main"]);

    b.iter(|| {
        vm.call(entry, (data.clone(),))
            .expect("successful execution")
    });

    Ok(())
}
