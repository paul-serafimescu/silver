#include <stdio.h>
#include <stdlib.h>
#include <ctype.h>
#include <string.h>
#include <unistd.h>

#include "request.h"

void clear_request(http_request *request);

http_request *create_request()
{
  http_request *request = malloc(sizeof(http_request));
  request->url = NULL;
  request->urlfull = NULL;
  request->qfields = NULL;
  request->headers.headers = NULL;
  request->body = NULL;
  return request;
}

int parse_request(FILE *socket_file, http_request *request)
{
  clear_request(request);
  request->socket_file = socket_file;

  char method_str[10];
  if (fscanf(socket_file, "%9s %ms HTTP/1.1\r\n", method_str, &request->urlfull) != 2) {
    return -1;
  }

  request->method = -1;
  for (size_t i = 0; i < REQUEST_METHODS; i++) {
    if (!strcmp(method_str, get_method_name(i))) {
      request->method = i;
      break;
    }
  }
  if (request->method == (request_method)-1) {
    return -1;
  }

  request->url = strdup(request->urlfull);
  char *querysep = strchr(request->url, '?');
  if (querysep) {
    *querysep = '\0';
    request->qfields = malloc(sizeof(request_qfield));
    request->qfields_size = 0;
    size_t qfields_capacity = 1;
    char *s = querysep + 1;
    int url_ended = 0;
    while (!url_ended && *s) {
      char *key_end = strpbrk(s, "=");
      if (!key_end) break;
      *key_end = '\0';
      size_t value_length = strcspn(key_end + 1, ";&");
      if (!*(key_end + 1 + value_length)) {
        url_ended = 1;
      }
      *(key_end + 1 + value_length) = '\0';
      if (request->qfields_size == qfields_capacity) {
        qfields_capacity *= 2;
        request->qfields =
          realloc(request->qfields, sizeof(request_qfield) * qfields_capacity);
      }
      request->qfields[request->qfields_size].key = s;
      request->qfields[request->qfields_size].value = key_end + 1;
      request->qfields_size++;
      s = key_end + 1 + value_length + 1;
    }
  } else {
    request->qfields = NULL;
    request->qfields_size = 0;
  }

  char *key;
  char *value;
  request->headers.headers = malloc(sizeof(http_header));
  request->headers.size = 0;
  request->headers.capacity = 1;
  while (fscanf(socket_file, "%m[^\r:]: %m[^\r]", &key, &value) == 2) {
    fgetc(socket_file);
    fgetc(socket_file);
    set_header(&request->headers, key, value);
  }
  fgetc(socket_file);
  fgetc(socket_file);

  if (request->method == REQUEST_POST ||
      request->method == REQUEST_PUT ||
      request->method == REQUEST_DELETE ||
      request->method == REQUEST_PATCH) {
    // No i don't know what a transfer encoding is
    char *content_length = get_header(&request->headers, "content-length");
    if (content_length) {
      request->body_size = atoi(content_length);
      request->body = malloc(request->body_size + 1);
      fread(request->body, 1, request->body_size, socket_file);
      request->body[request->body_size] = 0;
    }
  }

  return 0;
}

void clear_request(http_request *request)
{
  if (request->urlfull) {
    free(request->urlfull);
    request->urlfull = NULL;
  }
  if (request->url) {
    free(request->url);
    request->url = NULL;
  }
  if (request->qfields) {
    free(request->qfields);
    request->qfields = NULL;
  }
  if (request->headers.headers) {
    for (size_t i = 0; i < request->headers.size; i++) {
      free(request->headers.headers[i].key);
      free(request->headers.headers[i].value);
    }
    free(request->headers.headers);
    request->headers.headers = NULL;
  }
  if (request->body) {
    free(request->body);
    request->body = NULL;
  }
}

void destroy_request(http_request *request)
{
  clear_request(request);
  free(request);
}

char *get_request_qfield(const http_request *request, char *key)
{
  for (size_t i = 0; i < request->qfields_size; i++) {
    if (!strcmp(request->qfields[i].key, key)) {
      return request->qfields[i].value;
    }
  }
  return NULL;
}

const char *get_method_name(request_method method)
{
  switch (method) {
    case REQUEST_GET:
      return "GET";
    case REQUEST_HEAD:
      return "HEAD";
    case REQUEST_POST:
      return "POST";
    case REQUEST_PUT:
      return "PUT";
    case REQUEST_DELETE:
      return "DELETE";
    case REQUEST_PATCH:
      return "PATCH";
    default:
      return NULL;
  }
}

