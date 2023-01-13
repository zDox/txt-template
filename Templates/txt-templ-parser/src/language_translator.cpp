#include <vector>
#include "lexar.h"
#include "tokens.h"
#include "parser.h"
#include "include/rapidjson/document.h"
#include "include/base64/base64.h"
#include <iostream> 
#include <string> // the C++ Standard String Class
int main(void){
	std::string template_str = "lang: de \nend-of-settings!\nHallo ich bin ${name:Peter} Lustig.\nMein $elternteil ist doof";
	std::vector<Token *> tokens = lex(template_str);
	print_tokens(tokens);
	std::string pub = parse(tokens);
	std::cout << base64_encode(pub) << "\n";
	return 0;
}
