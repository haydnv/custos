use custos::{Buffer, CPU};

fn main() {
    let device = CPU::new();

    let a = Buffer::from((&device, [5, 7, 2, 10]));
    assert_eq!(a.read(), vec![5, 7, 2, 10])
}
