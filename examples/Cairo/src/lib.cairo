#[derive(Drop, Serde)]
struct Input {
    pub n: felt252
}

#[derive(Drop, Serde)]
struct Output {
    pub n: felt252
}

fn main(input: Array<felt252>) -> Array<felt252> {
    let mut input_span = input.span();
    let input = Serde::<Input>::deserialize(ref input_span).unwrap();

    let output = Output { n: fib(input.n) };

    let mut result = array![];
    output.serialize(ref result);
    result
}

fn fib(mut n: felt252) -> felt252 {
    let mut a: felt252 = 0;
    let mut b: felt252 = 1;
    while n != 0 {
        n = n - 1;
        let temp = b;
        b = a + b;
        a = temp;
    };

    a
}

#[cfg(test)]
mod tests {
    use super::fib;

    #[test]
    fn it_works() {
        assert(fib(16) == 987, 'it works!');
    }
}
