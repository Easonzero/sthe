#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum DescpType {
  Json,
  Toml,
} DescpType;

typedef enum RetCode {
  Succ,
  InvalidArgs,
} RetCode;

typedef struct ExtractOptCompiled ExtractOptCompiled;

enum RetCode compile_opt(const char *descp,
                         enum DescpType ty,
                         const struct ExtractOptCompiled **out);

void release_opt(struct ExtractOptCompiled *opt);

enum RetCode extract_fragment(const char *fragment,
                              const struct ExtractOptCompiled *opt,
                              enum DescpType ty,
                              const char **out);

enum RetCode extract_document(const char *document,
                              const struct ExtractOptCompiled *opt,
                              enum DescpType ty,
                              const char **out);

void release_extract(char *ret);
