#include <vector>
#include "tokens.h"
#include "utils.h"
#include "variables.h"
#include <string>
#include <algorithm>
#include <unordered_map>
#ifndef PARSER_H
#define PARSER_H
bool is_end_of_settings(Token* token){
	return token->type==TokenType_END_OF_SETTINGS;
}

Variable* get_variable(std::unordered_map<std::string, Variable*> &variable_storage, std::string identifier, VariableType type){
	if(variable_storage.find(identifier) != variable_storage.end()){
		Variable* variable = variable_storage.at(identifier);
		if (variable->type == type){
			return variable;
		}
	}
	return nullptr;
}

std::string parse(std::vector<Token *> tokens){
	std::unordered_map<std::string, Variable*> variable_storage;
	variable_storage["elternteil"] = new Variable(VariableType_CONSTANT, "Mutter");
	variable_storage["name"] = new Variable(VariableType_OPTION, "eltrnteil");
	
	bool settings_ended = std::find_if(tokens.begin(), tokens.end(), is_end_of_settings) == end(tokens);
	int cursor = 0;
	std::string str;
	while (!(settings_ended)){
		if(remove_whitespaces(tokens[cursor]->content) != "" && // Checks the following grammar without whitespaces text:text\n 
				tokens[cursor+1]->type == TokenType_COLON &&
				remove_whitespaces(tokens[cursor+2]->content) != "" &&
				tokens[cursor+3]->content == "\n"){
			variable_storage[remove_whitespaces(tokens[cursor]->content)] = new Variable(VariableType_SETTING, 
											remove_whitespaces(tokens[cursor+2]->content));
		}
		else if(tokens[cursor]->type == TokenType_END_OF_SETTINGS){
			settings_ended = true;
		}
		cursor++;
	}
	while (cursor < tokens.size()){
		if (tokens[cursor]->type == TokenType_DOLLER){
			if (	tokens[cursor+1]->type == TokenType_LBRACE && // DataType: Option
				tokens[cursor+2]->type == TokenType_TEXT &&
				((cursor+3 < tokens.size() &&
				 tokens[cursor+3]->type == TokenType_RBRACE) | // Without Default
				(cursor+5 < tokens.size() &&
				 tokens[cursor+3]->type == TokenType_COLON && // With Default
				 tokens[cursor+4]->type == TokenType_TEXT &&
				 tokens[cursor+5]->type == TokenType_RBRACE))){
				
				bool has_default = tokens[cursor+3]->type == TokenType_COLON;
				bool failed = false;
				std::string identifier = remove_whitespaces(tokens[cursor+2]->content);
				Variable* option = get_variable(variable_storage, identifier, VariableType_OPTION);
				
				if (option!=nullptr){
					Variable* constant = get_variable(variable_storage, option->value, VariableType_CONSTANT);
					if (constant!=nullptr){
						str += constant->value;
					}
					else if (has_default){ // The constant behind the option is not set so it now should get the default
						std::string default_str = remove_whitespaces(tokens[cursor+4]->content);
						str += default_str;
						std::cout << "Warning: Falling back to default. Option '" << identifier << "' can't get value for constant '" << option->value << "' not set" << "\n";
					}
					else {
						failed = true;
						std::cout << "Warning: Option '" << identifier << "' can't get value for constant '" << option->value << "' not set" << "\n";
					}
				}
				else if (has_default){ // The option is not set up so now should get the default
				      	std::string default_str = remove_whitespaces(tokens[cursor+4]->content);
					str += default_str;
					std::cout << "Warning: Falling back to default. Option '" << identifier << "' not set" << "\n";
				}
				else { // No default and the option has not been found -> failed
					failed = true; 
					std::cout << "Warning: Option '" << identifier << "' not set" << "\n";
				}

				if (failed) { // if it has to write the option statement as text and can't retrieve a value from the variable_storage
					for(int i =cursor; i< cursor + 4 + has_default*2; i++){
						str += tokens[i]->content;
					}
				}
				cursor += 3 + has_default *2;
			}
			else if(cursor+1 < tokens.size() &&
				tokens[cursor+1]->type == TokenType_TEXT){ // DataType: Constant
				std::string identifier;
				int index = tokens[cursor+1]->content.find(' '); // TO-DO: \n, \t should also be counted as whitespace
				if(index != std::string::npos && index != 0){ // After the identifier should come a whitespace
					identifier = tokens[cursor+1]->content.substr(0, index);
					Variable* constant = get_variable(variable_storage, identifier, VariableType_CONSTANT);
					if (constant != nullptr){

						str += constant->value;
						str += tokens[cursor+1]->content.substr(index, str.length()-index);
					}
					else {
						str += tokens[cursor]->content;
						str += tokens[cursor+1]->content;
						std::cout << "Warning: Constant '" << identifier << "' not set" << "\n";
					}
					cursor = cursor + 1;
				}
			}
		}
		else if(cursor+2 < tokens.size() && // VariableType: Key
			tokens[cursor]->type == TokenType_LBRACE &&
			tokens[cursor+1]->type == TokenType_TEXT &&
			tokens[cursor+2]->type == TokenType_RBRACE){
			
			std::string identifier = remove_whitespaces(tokens[cursor+1]->content);
			Variable* key = get_variable(variable_storage, identifier, VariableType_KEY);
			if (key!=nullptr){
				str += key->value;
			}
			else {
				str += tokens[cursor]->content;
				str += tokens[cursor+1]->content;
				str += tokens[cursor+2]->content;
				std::cout << "Warning: Key '" << identifier << "' not set" << "\n";
			}
			cursor = cursor + 2;

		}
		else if (tokens[cursor]->type == TokenType_TEXT){
			str += tokens[cursor]->content;
		}
		else {
			str += tokens[cursor]->content;
		}
		cursor++;
	}
	return str;
}
#endif
