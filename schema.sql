DROP TABLE IF EXISTS blog;

CREATE TABLE blog (
    slug TEXT PRIMARY KEY,
    title TEXT,
    published TEXT NOT NULL,
    last_updated TEXT,
    author TEXT,
    -- tags
    -- taxonomies? idk what that is even
    markdown_content TEXT
);

INSERT INTO blog VALUES (
    "2024-09-25-example-title",
    "example title",
    datetime(),
    NULL,
    "me",
    "# example blog post

what's up[^1]

[a link](https://grape.surgery/blog/asdf)

[^1]: this is a footnote...
"
);
