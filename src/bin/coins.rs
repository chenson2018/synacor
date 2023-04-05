use itertools::Itertools;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Clone)]
enum Coin {
    Red,
    Corroded,
    Shiny,
    Concave,
    Blue,
}

impl Coin {
    fn val(&self) -> u32 {
        match self {
            Self::Red => 2,
            Self::Corroded => 3,
            Self::Shiny => 5,
            Self::Concave => 7,
            Self::Blue => 9,
        }
    }
}

fn main() {
    let res = Coin::iter()
        .permutations(5)
        .find(|vec| match vec.as_slice() {
            [a, b, c, d, e] => {
                399 == a.val() + b.val() * u32::pow(c.val(), 2) + u32::pow(d.val(), 3) - e.val()
            }
            _ => unreachable!(),
        })
        .unwrap();

    println!("{:?}", res);
}
