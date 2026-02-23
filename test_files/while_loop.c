#include <stdio.h>

int main(void) {

printf("Enter a number:\n");
float X;
if(0 == scanf("%f", &X)) {
X = 0;
scanf("%*s");
}
while (((X)>(0))) {
printf("%.2f\n", (float)(X));
X = ((X)-(1));
}
printf("Done\n");
return 0;
}

