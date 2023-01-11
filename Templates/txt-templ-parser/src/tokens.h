#include <string>
#ifndef TOKENS_H
#define TOKENS_H
enum TokenType {
	TokenType_IDENTIFIER,
	TokenType_TEXT,
	TokenType_END_OF_SETTINGS,
	// Operators
	TokenType_COLON,

	// Seperators
	TokenType_LBRACE, // {
	TokenType_RBRACE, // }
	
	TokenType_DOLLER,
	TokenType_NEW_LINE
};
struct Token {
	TokenType type;
	std::string content;
	Token(TokenType new_type, std::string new_content){
		type = new_type;
		content = new_content;
	};
};


#endif
