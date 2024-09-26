DROP TABLE IF EXISTS blog;

CREATE TABLE blog (
    slug TEXT PRIMARY KEY NOT NULL,
    published DATETIME NOT NULL,
    title TEXT,
    last_updated DATETIME,
    author TEXT,
    draft BOOLEAN,
    -- tags
    -- taxonomies? idk what that is even
    markdown_content TEXT
);

INSERT INTO blog VALUES (
    "2024-09-25-example-title",
    datetime(),
    "example title",
    NULL,
    "me",
    FALSE,
    "# example blog post

what's[^unknown] up[^1]

[a link](https://grape.surgery/blog/asdf)

[a link later on][guh]

this para [has] [two] shortcuts?

# two

<zphixon@gmail.com>

[@another-blog-page]

hmm

[^1]: ref within footnote?[^hmm] maybe [link within footnote](google.com)
[^unreferenced]: bruh
[^hmm]: sqeaps

[guh]: https://egg.surgery/

[two]: /hm
"
);
