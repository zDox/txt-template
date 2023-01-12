#include <vector>
#include "lexar.h"
#include "tokens.h"
#include "parser.h"
#include <iostream> 
#include <string> // the C++ Standard String Class
int main(void){
	std::string template_str = "lang: de \nend-of-settings!\nHallo ich bin { name } Lustig.\nMein $elternteil ist doof";
	std::vector<Token *> tokens = lex(template_str);
	print_tokens(tokens);
	std::string pub = parse(tokens);
	std::cout << pub;
	return 0;
}
