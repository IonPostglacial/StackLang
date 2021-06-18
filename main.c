#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum TokenType {
    TOK_NUM,
    TOK_STR,
    TOK_SYM,
    TOK_ERR,
    TOK_END,
};

struct Token {
    enum TokenType type;
    size_t start;
    size_t end;
};

struct TokenStream {
    char* src;
    size_t len;
    struct Token tok;
};

void tok_stream_init(struct TokenStream* tokens, char* input)
{
    tokens->src = input;
    tokens->len = strlen(input);
    tokens->tok.type = TOK_ERR;
    tokens->tok.start = 0;
    tokens->tok.end = 0;
}

bool tok_stream_has_next(struct TokenStream tokens)
{
    return tokens.len > 0;
}

bool tok_stream_push(struct TokenStream* tokens, enum TokenType type,
    size_t start, size_t end)
{
    if (!tok_stream_has_next(*tokens)) {
        return false;
    }
    size_t len = end;
    tokens->tok.type = type;
    tokens->tok.start = tokens->tok.end + start;
    tokens->tok.end += end;
    tokens->len -= len;
    tokens->src += len;
    return true;
}

bool tok_stream_next(struct TokenStream* tokens)
{
    bool in_tok = false;
    bool in_str = false;
    bool in_num = false;
    size_t last_start = 0;
    for (size_t i = 0; i <= tokens->len; i++) {
        if (tokens->src[i] == '"') {
            in_str = !in_str;
            in_tok = in_str;
            if (in_str) {
                last_start = i;
            } else {
                return tok_stream_push(tokens, TOK_STR, last_start, i + 1);
            }
            continue;
        }
        bool is_sep = !in_str && (tokens->src[i] == ' ' || tokens->src[i] == '\t' || tokens->src[i] == '\0');
        if (is_sep && in_tok) {
            return tok_stream_push(tokens, in_num ? TOK_NUM : TOK_SYM, last_start, i);
        }
        if (!is_sep && !in_tok) {
            last_start = i;
            in_num = tokens->src[i] >= '0' && tokens->src[i] <= '9';
        }
        in_tok = !is_sep;
    }
    if (in_str) {
        return tok_stream_push(tokens, TOK_ERR, last_start, tokens->len);
    }
    return tok_stream_push(tokens, TOK_END, tokens->len, tokens->len + 1);
}

enum StackCellType {
    CELL_TYPE_NUM,
    CELL_TYPE_STR,
};

enum StackStatus {
    STACK_OK,
    STACK_OVERFLOW,
    STACK_UNDERFLOW,
};

enum KnownSymbol {
    SYM_NOP,
    SYM_ADD,
    SYM_SUB,
    SYM_MUL,
    SYM_DIV,
};

struct StackCell {
    enum StackCellType type;
    union {
        double num;
        char* str;
    };
};

struct StackMachine {
    struct StackCell* stack;
    size_t cap;
    size_t sp;
};

void stack_machine_init(struct StackMachine* machine, struct StackCell* stack, size_t cap)
{
    machine->stack = stack;
    machine->cap = cap;
    machine->sp = 0;
    memset(machine->stack, 0, cap);
}

enum StackStatus stack_machine_push(struct StackMachine* machine, struct StackCell cell)
{
    if (machine->sp >= machine->cap - 1) {
        return STACK_OVERFLOW;
    } else {
        machine->sp++;
        machine->stack[machine->sp] = cell;
    }
    return STACK_OK;
}

enum StackStatus stack_machine_pop(struct StackMachine* machine)
{
    if (machine->sp <= 0) {
        return STACK_UNDERFLOW;
    } else {
        machine->sp--;
    }
    return STACK_OK;
}

struct StackCell stack_machine_peek(struct StackMachine* machine)
{
    if (machine->sp < 0) {
        struct StackCell cell;
        cell.type = CELL_TYPE_NUM;
        cell.num = 0;
        return cell;
    }
    return machine->stack[machine->sp];
}

enum StackStatus stack_machine_exec_sym(struct StackMachine* machine, enum KnownSymbol sym)
{
    enum StackStatus status = STACK_OK;
    struct StackCell op1, op2;
    struct StackCell res;

    switch (sym) {
    case SYM_ADD:
        op1 = stack_machine_peek(machine);
        stack_machine_pop(machine);
        op2 = stack_machine_peek(machine);
        status = stack_machine_pop(machine);
        if (status == STACK_OK && op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            res.type = CELL_TYPE_NUM;
            res.num = op1.num + op2.num;
            stack_machine_push(machine, res);
        }
        break;
    case SYM_SUB:
        op1 = stack_machine_peek(machine);
        stack_machine_pop(machine);
        op2 = stack_machine_peek(machine);
        status = stack_machine_pop(machine);
        if (status == STACK_OK && op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            res.type = CELL_TYPE_NUM;
            res.num = op1.num - op2.num;
            stack_machine_push(machine, res);
        }
        break;
    case SYM_MUL:
        op1 = stack_machine_peek(machine);
        stack_machine_pop(machine);
        op2 = stack_machine_peek(machine);
        status = stack_machine_pop(machine);
        if (status == STACK_OK && op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            res.type = CELL_TYPE_NUM;
            res.num = op1.num * op2.num;
            stack_machine_push(machine, res);
        }
        break;
    case SYM_DIV:
        op1 = stack_machine_peek(machine);
        stack_machine_pop(machine);
        op2 = stack_machine_peek(machine);
        status = stack_machine_pop(machine);
        if (status == STACK_OK && op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            res.type = CELL_TYPE_NUM;
            res.num = op1.num / op2.num;
            stack_machine_push(machine, res);
        }
        break;
    }
    return status;
}

enum StackStatus stack_machine_eval(struct StackMachine* machine, char* input)
{
    struct TokenStream tokens;
    enum StackStatus status = STACK_OK;
    struct StackCell cell;
    enum KnownSymbol sym;
    char* tokend;
    tok_stream_init(&tokens, input);
    while (tok_stream_next(&tokens)) {
        switch (tokens.tok.type) {
        case TOK_NUM:
            cell.type = CELL_TYPE_NUM;
            cell.num = strtod(&input[tokens.tok.start], &tokend);
            status = stack_machine_push(machine, cell);
            break;
        case TOK_SYM:
            if (input[tokens.tok.start] == '+') {
                sym = SYM_ADD;
            } else if (input[tokens.tok.start] == '-') {
                sym = SYM_SUB;
            } else if (input[tokens.tok.start] == '*') {
                sym = SYM_MUL;
            } else if (input[tokens.tok.start] == '/') {
                sym = SYM_DIV;
            } else {
                sym = SYM_NOP;
            }
            status = stack_machine_exec_sym(machine, sym);
            break;
        }
    }
    return status;
}

int main(int argc, char* argv[])
{
    if (argc < 2) {
        return 0;
    } else {
        char* input = argv[1];

        struct StackMachine machine;
        struct StackCell stack[255];

        stack_machine_init(&machine, stack, 255);
        enum StackStatus status = stack_machine_eval(&machine, input);

        printf("stack { status: %d, sp: %ld, top: %lf }\n", status, machine.sp, stack_machine_peek(&machine).num);
    }
}
