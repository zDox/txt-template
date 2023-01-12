#include <string>
#include <unordered_map>
#ifndef VARIABLES_H
#define VARIABLES_H


enum VariableType {
	VariableType_SETTING,
	VariableType_CONSTANT,
	VariableType_OPTION,
	VariableType_KEY
};

struct Variable {
	VariableType type;
	std::string value;
	Variable (VariableType new_type, std::string new_value){
		type = new_type;
		value = new_value;
	}
};

#endif
