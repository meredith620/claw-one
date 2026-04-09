/// Test module for basic Rust operations
///
/// This module contains simple unit tests validating core Rust functionality:
/// - Basic arithmetic
/// - String operations
/// - Vector operations

#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic integer addition
    #[test]
    fn test_simple_addition() {
        let result = 2 + 2;
        assert_eq!(result, 4, "2 + 2 should equal 4");
    }

    /// Test string concatenation
    #[test]
    fn test_string_concatenation() {
        let hello = String::from("Hello, ");
        let world = String::from("world!");
        let combined = hello + &world;
        assert_eq!(combined, "Hello, world!");

        // Also test &str concatenation with &
        let s1 = String::from("foo");
        let s2 = String::from("bar");
        let s3 = format!("{}{}", s1, s2);
        assert_eq!(s3, "foobar");
    }

    /// Test Vec push and pop operations
    #[test]
    fn test_vector_operations() {
        let mut v: Vec<i32> = Vec::new();

        // Test push
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[2], 3);

        // Test pop
        let popped = v.pop();
        assert_eq!(popped, Some(3));
        assert_eq!(v.len(), 2);

        let popped2 = v.pop();
        assert_eq!(popped2, Some(2));
        assert_eq!(v.len(), 1);

        // After popping all elements
        let empty_pop = v.pop();
        assert_eq!(empty_pop, Some(1));
        assert_eq!(v.len(), 0);

        // Pop from empty vec returns None
        let none_pop = v.pop();
        assert_eq!(none_pop, None);
    }
}
