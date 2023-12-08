<!-- Bolded words are terminals, lowercase terminals are keywords -->

<!-- START SYMBOL --->
program -> node program
program -> node

node -> node_header *{* stmts *}*
node_header -> *node* *IDENTIFIER* node_list
node_list -> *`*
node_list -> *:* param_list

param_list -> param
param_list -> param *,* param_list
param -> *IDENTIFIER*
param -> expression

stmts -> stmt *;*
stmts -> stmt *;* stmts

expression -> term rest
rest -> operator expression rest
rest -> *`*

term -> *NUMBER*
term -> *IDENTIFIER*

operator -> *+*
operator -> *-*