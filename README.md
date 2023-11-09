# Calculator
Calculator is a library which support calculates the value of string.
## Usage
```rust
fn main(){
    {
        let calculator = "1+1".calculate();
        assert_eq!(calculator, Ok(Value::Integer(2)))
    }

    {
        let calculator = "1*1".calculate();
        assert_eq!(calculator, Ok(Value::Integer(1)))
    }

    {
        let calculator = "2*4".calculate();
        assert_eq!(calculator, Ok(Value::Integer(8)))
    }

    {
        let calculator = "4!".calculate();
        assert_eq!(calculator, Ok(Value::Integer(24)))
    }

    {
        let calculator = "31%15".calculate();
        assert_eq!(calculator, Ok(Value::Integer(1)))
    }

    {
        let calculator = "1*!1".calculate();
        assert!(calculator.is_err())
    }
}
```
## Inspired
- [toydb](https://github.com/erikgrinaker/toydb/tree/master/src/sql/parser)