
def hello_world():
    """A simple function."""
    print("Hello, World!")

def calculate_sum(a, b):
    """Calculate sum of two numbers."""
    return a + b

class Calculator:
    def __init__(self):
        self.value = 0
    
    def add(self, x):
        self.value += x
        return self
