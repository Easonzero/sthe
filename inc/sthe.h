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

typedef struct ExtractOptCompled ExtractOptCompled;

enum RetCode compile_opt(const char *_descp,
                         enum DescpType _ty,
                         const struct ExtractOptCompled **_opt);

void release_opt(const struct ExtractOptCompled *_opt);

const char *extract_fragment(const char *_fragment,
                             const struct ExtractOptCompled *_opt,
                             enum DescpType _ty);

const char *extract_document(const char *_document,
                             const struct ExtractOptCompled *_opt,
                             enum DescpType _ty);

void release_extract(const char *_ret);
