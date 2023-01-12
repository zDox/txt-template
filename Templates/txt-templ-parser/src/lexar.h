#include <vector>
#include <iostream>
#include <string>
#include "tokens.h"
#include <algorithm>
#ifndef LEXAR_H
#define LEXAR_H
const char whitespaces[] = {'\t', ' '};
const std::string end_settings_block = "end-of-settings!";
const char punctuators[] = {':', '{', '}', '$'};

bool is_whitespace(char c){
	return (std::find(std::begin(whitespaces), std::end(whitespaces), c) != std::end(whitespaces));
}
bool is_punctuator(char c){
	return (std::find(std::begin(punctuators), std::end(punctuators), c) != std::end(punctuators));
}
std::vector<Token *> lex(const std::string &str){	
	std::vector<Token *> tokens;	
	int left = 0, right = 0;
	int length = str.length();
	bool settings_decleared = str.find(end_settings_block) != std::string::npos;
	std::cout << settings_decleared<<'\n';
	
	while(right<length && left<=right){
		std::cout << "l: " << left << " r: " << right << '\n';
		if(settings_decleared){
			if(left==right && is_whitespace(str.at(right))){
				right++;
				left=right;
			}
			else if(left==right && str.at(right) == ':') {
				right++;
				left=right;
				tokens.push_back(new Token(TokenType_COLON, ":"));
			}
			else if(left==right && str.at(right) == '\n'){
				right++;
				left=right;
				tokens.push_back(new Token(TokenType_NEW_LINE, "\n"));
			}	
			else if(left!=right && (str.at(right) == ':' | is_whitespace(str.at(right)) | str.at(right) == '\n')){
				std::string sub = str.substr(left, right-left);
				if (sub.compare(end_settings_block)== 0){
					tokens.push_back(new Token(TokenType_END_OF_SETTINGS, end_settings_block));
					settings_decleared = false;
					right = str.find('\n', right)+1;
				}
				else {
					tokens.push_back(new Token(TokenType_TEXT, sub));
				}
				left=right;
			}
			else if(!(str.at(right) == ':' | is_whitespace(str.at(right)) | str.at(right) == '\n')){
				right++;
			}	
		}
		else {
			if(is_punctuator(str.at(right)) && left!=right){
				std::string sub = str.substr(left, right-left);
				tokens.push_back(new Token(TokenType_TEXT, sub));
				left = right;
			}
			else if (is_punctuator(str.at(right)) && left == right){
				switch (str.at(right)){
					case '{':
						tokens.push_back(new Token(TokenType_LBRACE, "{"));
						break;
					case '}':
						tokens.push_back(new Token(TokenType_RBRACE, "}"));
						break;
					case '$':
						tokens.push_back(new Token(TokenType_DOLLER, "$"));
						break;
					case ':':
						tokens.push_back(new Token(TokenType_COLON, ":"));
						break;
				}
				right++;
				left=right;
			}
			else {
				right++;
				if (right >= length){
					std::string sub = str.substr(left, right-left);
					tokens.push_back(new Token(TokenType_TEXT, sub));
				}
			}
		}
	}
	return tokens;
}

void print_tokens(std::vector<Token *>tokens){
	for (int i = 0; i < tokens.size(); i++){
		std::cout << i << ". content: " << tokens[i]->content << '\n';
	}
}	
#endif
