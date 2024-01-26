import math
def calc(n):
    for i in range(0,n):
        m = math.log2(11*i+3)
        if int(m) == m:
            print("i = ",i, "m = ", m)
            
            
def calc2(n):
    for i in range(0,n):
        m8 = 10*i+6
        if (2**m8) % 11 == 9:
            print("i = ",i, "m8 = ", m8) 
            
calc2(20)