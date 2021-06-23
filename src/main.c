#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef enum {
    TOK_NUM,
    TOK_STR,
    TOK_SYM,
    TOK_ERR,
    TOK_END,
} TokenType;

typedef struct {
    TokenType type;
    size_t start;
    size_t end;
} Token;

typedef struct {
    char* src;
    size_t len;
    Token tok;
} TokenStream;

void tok_stream_init(TokenStream* tokens, char* input)
{
    tokens->src = input;
    tokens->len = strlen(input);
    tokens->tok.type = TOK_ERR;
    tokens->tok.start = 0;
    tokens->tok.end = 0;
}

bool tok_stream_has_next(TokenStream tokens)
{
    return tokens.len > 0;
}

bool tok_stream_push(TokenStream* tokens, TokenType type,
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

bool tok_stream_next(TokenStream* tokens)
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

typedef enum {
    CELL_TYPE_ERR,
    CELL_TYPE_NUM,
    CELL_TYPE_STR,
} StackCellType;

typedef enum {
    STACK_ERR_NONE,
    STACK_ERR_OVERFLOW,
    STACK_ERR_UNDERFLOW,
} StackError;

typedef enum {
    SYM_NOP,
    SYM_ADD,
    SYM_SUB,
    SYM_MUL,
    SYM_DIV,
    SYM_POP,
    SYM_DUP,
    SYM_INC,
    SYM_DEC,
} KnownSymbol;

typedef struct {
    StackCellType type;
    union {
        double num;
        StackError err;
        char* str;
    } as;
} StackCell;

typedef struct {
    StackCell* stack;
    size_t cap;
    size_t sp;
} StackMachine;

void stack_machine_init(StackMachine* machine)
{
    machine->cap = 256;
    machine->stack = malloc(machine->cap * sizeof(StackCell));
    machine->stack[0].type = CELL_TYPE_ERR;
    machine->stack[0].as.err = STACK_ERR_UNDERFLOW;
    machine->sp = 0;
}

void stack_machine_free(StackMachine* machine)
{
    free(machine->stack);
    machine->stack = NULL;
    machine->cap = 0;
}

StackError stack_machine_push(StackMachine* machine, StackCell cell)
{
    if (machine->sp >= machine->cap - 1) {
        machine->cap *= 2;
        StackCell* newstack = realloc(machine->stack, sizeof(StackCell) * machine->cap);
        if (newstack == NULL) {
            stack_machine_free(machine);
            return STACK_ERR_OVERFLOW;
        } else {
            machine->stack = newstack;
        }
    }
    machine->sp++;
    machine->stack[machine->sp] = cell;
    return STACK_ERR_NONE;
}

StackCell stack_machine_pop(StackMachine* machine)
{
    StackCell top = machine->stack[machine->sp];
    if (machine->sp > 0) {
        machine->sp--;
    }
    return top;
}

StackCell stack_machine_peek(StackMachine* machine)
{
    return machine->stack[machine->sp];
}

StackError stack_machine_exec_sym(StackMachine* machine, KnownSymbol sym)
{
    StackCell op1, op2;

    switch (sym) {
    case SYM_ADD:
        op1 = stack_machine_pop(machine);
        op2 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_NUM, .as = { .num = op1.as.num + op2.as.num } });
        }
        break;
    case SYM_SUB:
        op1 = stack_machine_pop(machine);
        op2 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_NUM, .as = { .num = op1.as.num - op2.as.num } });
        }
        break;
    case SYM_MUL:
        op1 = stack_machine_pop(machine);
        op2 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_NUM, .as = { .num = op1.as.num * op2.as.num } });
        }
        break;
    case SYM_DIV:
        op1 = stack_machine_pop(machine);
        op2 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_NUM, .as = { .num = op1.as.num / op2.as.num } });
        }
        break;
    case SYM_POP:
        stack_machine_pop(machine);
        break;
    case SYM_DUP:
        stack_machine_push(machine, stack_machine_peek(machine));
        break;
    case SYM_INC:
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM) {
            stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_NUM, .as = { .num = op1.as.num + 1 } });
        }
        break;
    case SYM_DEC:
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM) {
            stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_NUM, .as = { .num = op1.as.num - 1 } });
        }
        break;
    case SYM_NOP:
        break;
    }
    StackCell top = stack_machine_peek(machine);
    if (top.type == CELL_TYPE_ERR) {
        return top.as.err;
    } else {
        return STACK_ERR_NONE;
    }
}

StackError stack_machine_eval(StackMachine* machine, char* input)
{
    TokenStream tokens;
    StackError err = STACK_ERR_NONE;
    StackCell cell;
    KnownSymbol sym;
    char* tokend;
    tok_stream_init(&tokens, input);
    while (tok_stream_next(&tokens)) {
        size_t toklen = tokens.tok.end - tokens.tok.start;
        switch (tokens.tok.type) {
        case TOK_END:
            return STACK_ERR_NONE;
        case TOK_NUM:
            cell.type = CELL_TYPE_NUM;
            cell.as.num = strtod(&input[tokens.tok.start], &tokend);
            err = stack_machine_push(machine, cell);
            break;
        case TOK_SYM:
            if (toklen == 1 && input[tokens.tok.start] == '+') {
                sym = SYM_ADD;
            } else if (toklen == 1 && input[tokens.tok.start] == '-') {
                sym = SYM_SUB;
            } else if (toklen == 1 && input[tokens.tok.start] == '*') {
                sym = SYM_MUL;
            } else if (toklen == 1 && input[tokens.tok.start] == '.') {
                sym = SYM_POP;
            } else if (toklen == 3 && memcmp(&input[tokens.tok.start], "dup", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_DUP;
            } else if (toklen == 1 && input[tokens.tok.start] == '/') {
                sym = SYM_DIV;
            } else if (toklen == 3 && memcmp(&input[tokens.tok.start], "inc", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_INC;
            } else if (toklen == 3 && memcmp(&input[tokens.tok.start], "dec", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_DEC;
            } else {
                sym = SYM_NOP;
            }
            err = stack_machine_exec_sym(machine, sym);
            break;
        }
    }
    return err;
}

int main(int argc, char* argv[])
{
    if (argc < 2) {
        return 0;
    } else {
        char* input = argv[1];

        StackMachine machine;

        stack_machine_init(&machine);
        StackError err = stack_machine_eval(&machine, input);
        switch (err) {
        case STACK_ERR_NONE:
            for (size_t i = machine.sp; i > 0; i--) {
                printf("%ld\t%f\n", i, machine.stack[i].as.num);
            }
            break;
        case STACK_ERR_OVERFLOW:
            puts("stack overflow");
            break;
        case STACK_ERR_UNDERFLOW:
            puts("stack underflow");
            break;
        }
        stack_machine_free(&machine);
    }
}
