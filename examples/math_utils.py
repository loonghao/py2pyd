"""
Math utilities module for testing py2pyd_rs.
"""

def add(a, b):
    """Add two numbers."""
    return a + b

def subtract(a, b):
    """Subtract b from a."""
    return a - b

def multiply(a, b):
    """Multiply two numbers."""
    return a * b

def divide(a, b):
    """Divide a by b."""
    if b == 0:
        raise ValueError("Cannot divide by zero")
    return a / b

class Vector3D:
    """A 3D vector class."""
    
    def __init__(self, x=0, y=0, z=0):
        """Initialize a 3D vector."""
        self.x = x
        self.y = y
        self.z = z
    
    def magnitude(self):
        """Calculate the magnitude of the vector."""
        return (self.x**2 + self.y**2 + self.z**2)**0.5
    
    def normalize(self):
        """Normalize the vector."""
        mag = self.magnitude()
        if mag == 0:
            return Vector3D(0, 0, 0)
        return Vector3D(self.x/mag, self.y/mag, self.z/mag)
    
    def dot(self, other):
        """Calculate the dot product with another vector."""
        return self.x * other.x + self.y * other.y + self.z * other.z
    
    def cross(self, other):
        """Calculate the cross product with another vector."""
        return Vector3D(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x
        )
