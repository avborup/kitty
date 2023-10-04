from sys import stdin, stderr

x, y = int(next(stdin)), int(next(stdin))

print(f"Input was ({x}, {y})", file=stderr)

if x < 0 and y < 0:
    print(1)
if x < 0 and y > 0:
    print(2)
if x > 0 and y > 0:
    print(3)
if x > 0 and y < 0:
    print(4)
