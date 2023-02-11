from sys import stdin

x, y = int(next(stdin)), int(next(stdin))

if x > 0 and y > 0:
    print(1)
if x < 0 and y > 0:
    print(2)
if x < 0 and y < 0:
    print(3)
if x > 0 and y < 0:
    print(4)
