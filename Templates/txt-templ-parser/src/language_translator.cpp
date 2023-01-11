#include <vector>
#include "lexar.h"
#include "tokens.h"
#include <iostream> 
#include <string> // the C++ Standard String Class
int main() {
	std::string template_str = "lang: de \nend-of-settings!\nHallo ich bin {name:Hi} Lustig";
	std::vector<Token *> tokens = lex(template_str);
	print_tokens(tokens);
	return 0;
}
