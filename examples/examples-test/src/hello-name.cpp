#include <iostream>
#include <string>

int main() {
    std::string name;

    std::cout << "Please enter your name > ";
    std::cin >> name;

    std::cout << "Hello " + name + "!" << std::endl;
}
