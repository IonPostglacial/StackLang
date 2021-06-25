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
    CELL_TYPE_BOOL,
} StackCellType;

typedef enum {
    STACK_ERR_NONE,
    STACK_ERR_OVERFLOW,
    STACK_ERR_UNDERFLOW,
    STACK_ERR_TYPE,
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
    SYM_TRUE,
    SYM_FALSE,
    SYM_EQ,
    SYM_NOT,
    SYM_AND,
    SYM_OR,
} KnownSymbol;

typedef struct {
    StackCellType type;
    union {
        double num;
        StackError err;
        char* str;
        bool boolean;
    } as;
} StackCell;

bool stack_cell_eq(StackCell c1, StackCell c2)
{
    return c1.type == c2.type &&
        (c1.type == CELL_TYPE_NUM ) ? (c1.as.num     == c2.as.num    ) :
        (c1.type == CELL_TYPE_BOOL) ? (c1.as.boolean == c2.as.boolean) :
        (c1.type == CELL_TYPE_STR ) ? (strcmp(c1.as.str, c2.as.str) == 0) :
        false;
}

typedef struct {
    StackCell* stack;
    size_t cap;
    size_t sp;
} StackMachine;

void stack_machine_init(StackMachine* machine)
{
    machine->cap = 64;
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

StackError stack_machine_push_num(StackMachine *machine, double num)
{
    return stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_NUM, .as = { .num = num } });
}

StackError stack_machine_push_err(StackMachine *machine, StackError err)
{
    return stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_ERR, .as = { .err = err } });
}

StackError stack_machine_push_bool(StackMachine *machine, bool b)
{
    return stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_BOOL, .as = { .boolean = b } });
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
        op2 = stack_machine_pop(machine);
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            stack_machine_push_num(machine, op1.as.num + op2.as.num);
        } else if (op2.type != CELL_TYPE_ERR) {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
        }
        break;
    case SYM_SUB:
        op2 = stack_machine_pop(machine);
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            stack_machine_push_num(machine, op1.as.num - op2.as.num);
        } else if (op2.type != CELL_TYPE_ERR) {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
        }
        break;
    case SYM_MUL:
        op2 = stack_machine_pop(machine);
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            stack_machine_push_num(machine, op1.as.num * op2.as.num);
        } else {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
        }
        break;
    case SYM_DIV:
        op2 = stack_machine_pop(machine);
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM && op2.type == CELL_TYPE_NUM) {
            stack_machine_push_num(machine, op1.as.num / op2.as.num);
        } else if (op2.type != CELL_TYPE_ERR) {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
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
            stack_machine_push_num(machine, op1.as.num + 1);
        } else if (op1.type != CELL_TYPE_ERR) {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
        }
        break;
    case SYM_DEC:
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_NUM) {
            stack_machine_push_num(machine, op1.as.num - 1);
        } else if (op1.type != CELL_TYPE_ERR) {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
        }
        break;
    case SYM_TRUE:
        stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_BOOL, .as = { .boolean = true } });
        break;
    case SYM_FALSE:
        stack_machine_push(machine, (StackCell) { .type = CELL_TYPE_BOOL, .as = { .boolean = false } });
        break;
    case SYM_EQ:
        op2 = stack_machine_pop(machine);
        op1 = stack_machine_pop(machine);
        if (op1.type != CELL_TYPE_ERR && op2.type != CELL_TYPE_ERR) {
            stack_machine_push_bool(machine, stack_cell_eq(op1, op2));
        }
        break;
    case SYM_NOT:
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_BOOL) {
            stack_machine_push_bool(machine, !op1.as.boolean);
        } else if (op1.type != CELL_TYPE_ERR) {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
        }
        break;
    case SYM_AND:
        op2 = stack_machine_pop(machine);
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_BOOL && op2.type == CELL_TYPE_BOOL) {
            stack_machine_push_bool(machine, op1.as.boolean && op2.as.boolean);
        } else if (op2.type != CELL_TYPE_ERR) {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
        }
        break;
    case SYM_OR:
        op2 = stack_machine_pop(machine);
        op1 = stack_machine_pop(machine);
        if (op1.type == CELL_TYPE_BOOL && op2.type == CELL_TYPE_BOOL) {
            stack_machine_push_bool(machine, op1.as.boolean || op2.as.boolean);
        } else if (op2.type != CELL_TYPE_ERR) {
            stack_machine_push_err(machine, STACK_ERR_TYPE);
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
    while (err == STACK_ERR_NONE && tok_stream_next(&tokens)) {
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
            } else if (toklen == 4 && memcmp(&input[tokens.tok.start], "true", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_TRUE;
            } else if (toklen == 5 && memcmp(&input[tokens.tok.start], "false", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_FALSE;
            } else if (toklen == 1 && memcmp(&input[tokens.tok.start], "=", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_EQ;
            } else if (toklen == 3 && memcmp(&input[tokens.tok.start], "not", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_NOT;
            } else if (toklen == 3 && memcmp(&input[tokens.tok.start], "and", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_AND;
            } else if (toklen == 2 && memcmp(&input[tokens.tok.start], "or", tokens.tok.end - tokens.tok.start) == 0) {
                sym = SYM_OR;
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
                if (machine.stack[i].type == CELL_TYPE_NUM) {
                    printf("%lu\t%f\n", (long unsigned int)i, machine.stack[i].as.num);
                } else if (machine.stack[i].type == CELL_TYPE_BOOL) {
                    printf("%lu\t%s\n", (long unsigned int)i, machine.stack[i].as.boolean ? "true" : "false");
                }
            }
            break;
        case STACK_ERR_OVERFLOW:
            puts("stack overflow");
            break;
        case STACK_ERR_UNDERFLOW:
            puts("stack underflow");
            break;
        case STACK_ERR_TYPE:
            puts("type error");
            break;
        }
        stack_machine_free(&machine);
    }
}
