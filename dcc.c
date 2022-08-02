#include <ctype.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// トークンの種類

/*
enum タグ名 { members } と定義することで、
enum タグ名 var; のように「enum タグ名」型を作成することができる

typedef 型名 新しい型名 と定義することで型名を新しい型名としても使用することができるようになる

typedefとenumを合わせる際は、enumのタグ名を省略することができる
*/
typedef enum
{
  TK_RESERVED, // 記号
  TK_NUM,      // 整数トークン
  TK_EOF,      // 入力の終わりを表すトークン
} TokenKind;

typedef struct Token Token;

struct Token
{
  TokenKind kind;
  Token *next; // 次の入力トークン
  int val;     // TK_NUMの場合その数値
  char *str;   // トークンの文字列
};

// 現在着目しているトークン
Token *token;
// 入力プログラム
char *user_input;

// エラー報告用の関数
// printfと同じ引数を取る
// va_listは可変長の引数を受け取る(stdarg.h)
void error(char *fmt, ...)
{
  va_list ap;
  va_start(ap, fmt);
  vfprintf(stderr, fmt, ap);
  fprintf(stderr, "\n");
  exit(1);
}

// エラーの箇所を報告する
void error_at(char *loc, char *fmt, ...)
{
  va_list ap;
  va_start(ap, fmt);
  int pos = loc - user_input;
  fprintf(stderr, "%s\n", user_input);
  fprintf(stderr, "%*s", pos, " "); // pos個の空白を出力
  fprintf(stderr, "^ ");
  vfprintf(stderr, fmt, ap);
  fprintf(stderr, "\n");
  exit(1);
}

// 次のトークンが期待している記号になっているときは、トークンを1つ読み進めてtrueを返す。それ以外にはfalseを返す。
bool consume(char op)
{
  if (!(token->kind == TK_RESERVED && token->str[0] == op))
  {
    return false;
  }
  token = token->next;
  return true;
}

// 次のトークンが期待している記号になっているときは、トークンを1つ読み進める。それ以外にはエラーを報告する。
void expect(char op)
{
  if (!(token->kind == TK_RESERVED && token->str[0] == op))
  {
    error_at(token->str, "'%c'ではありません。", op);
  }
  token = token->next;
}

// 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。
int expect_number()
{
  if (token->kind != TK_NUM)
  {
    error_at(token->str, "数値ではありません。");
  }
  int val = token->val;
  token = token->next;
  return val;
}

// eofにいるか判定
bool at_eof()
{
  return token->kind == TK_EOF;
}

// 新しいトークンを作成してcurに繋げる
Token *new_token(TokenKind kind, Token *cur, char *str)
{
  // 0埋めのためcallocを使っている
  Token *tok = calloc(1, sizeof(Token));
  tok->kind = kind;
  tok->str = str;
  cur->next = tok;
  return tok;
}

// 入力文字列pをトークナイズしてそれを返す

// tokenize関数では連結リストを構築しています。連結リストを構築するときは、ダミーのhead要素を作ってそこに新しい要素を繋げていって、最後にhead->nextを返すようにするとコードが簡単になります。このような方法ではhead要素に割り当てられたメモリはほとんど無駄になりますが、ローカル変数をアロケートするコストはほぼゼロなので、特に気にする必要はありません。
Token *tokenize(char *p)
{
  Token head;
  head.next = NULL;
  Token *cur = &head;

  while (*p)
  {
    // 空白文字をスキップ
    if (isspace(*p))
    {
      p++;
      continue;
    }

    if (*p == '+' || *p == '-')
    {
      cur = new_token(TK_RESERVED, cur, p++);
      continue;
    }

    if (isdigit(*p))
    {
      cur = new_token(TK_NUM, cur, p);
      cur->val = strtol(p, &p, 10);
      continue;
    }

    error_at(cur->str, "トークナイズできません。");
  }

  new_token(TK_EOF, cur, p);
  return head.next;
}

int main(int argc, char **argv)
{
  if (argc != 2)
  {
    fprintf(stderr, "引数の個数が正しくありません。\n");
    return 1;
  }

  // トークナイズする
  user_input = argv[1];
  token = tokenize(argv[1]);

  printf(".intel_syntax noprefix\n");
  printf(".globl main\n");
  printf("main:\n");

  // 式の最初は数値であるはず
  printf("  mov rax, %d\n", expect_number());

  // `+ <数>`あるいは`- <数>`というトークンの並びを消費しつつアセンブリを出力
  while (!at_eof())
  {
    if (consume('+'))
    {
      printf("  add rax, %d\n", expect_number());
      continue;
    }

    expect('-');
    printf("  sub rax, %d\n", expect_number());
  }

  printf("  ret\n");
  return 0;
}