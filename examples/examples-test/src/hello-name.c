#include <stdio.h>

int main(void) {
    char name[20];

    printf("Please enter your name > ");
    scanf("%s", name);

    printf("Hello %s!\n", name);
}
