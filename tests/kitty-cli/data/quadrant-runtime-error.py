from sys import stdin

x, y = int(next(stdin)), int(next(stdin))

if x > 0 and y > 0:
    raise Exception("I don't know what quadrant this is in!")
if x < 0 and y > 0:
    print(2)
if x < 0 and y < 0:
    print(3)
if x > 0 and y < 0:
    print(4)
