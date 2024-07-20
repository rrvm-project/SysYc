
int main()
{
    int ans ;
    int i ;
    int x ;
    int y ;

    int ml = 128, mr = 256, mres = 0;
    while (mr)
    {
        putch(99);
        ans = 0;
        i = 0;
        x = mr;

        while (i < 15)
        {
            if (x % 2 )
            {
                ans = ans + 1;
            }
            i = i + 1;
        }

        mr = ans;
    }

    return ans;
}