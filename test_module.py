def hello_world():
    """A simple hello world function."""
    return "Hello, World!"

def add_numbers(a, b):
    """Add two numbers together."""
    return a + b

class Calculator:
    """A simple calculator class."""
    
    def __init__(self):
        self.result = 0
    
    def add(self, value):
        """Add a value to the result."""
        self.result += value
        return self.result
    
    def multiply(self, value):
        """Multiply the result by a value."""
        self.result *= value
        return self.result

if __name__ == "__main__":
    print(hello_world())
    print(add_numbers(5, 3))
    
    calc = Calculator()
    calc.add(10)
    calc.multiply(2)
    print(f"Calculator result: {calc.result}")
