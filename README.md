`preimage`: find what hashes to a given value
=============================================

The `preimage` utility maintains a database of hash-preimages. It will scan your
hard drive for "hashable** content, compute the hashes, then store references to
where it can be found.

**Why?** Rather than link a file by its location, you can **link to its content**.
This means that when a file does not change&mdash;for example, because you saved
a copy of a document that you sent to someone&mdash;*the link will not break if
the file is moved*. Conversely, the link _will_ break if the file _contents_
change, meaning that you can be certain that you are looking at the right
version of the file.

We store two kinds of hash:

  + **File hashes.**  The SHA-256 hash of a file's contents.
  + **Git commit hashes.**  The SHA-1 identifier of a `git` commit.

Some other hashable objects that are planned for future releases:

  + Email attachments.
  + Files stored in of zipfiles/tarballs/etc.
  + Historical file contents from `git` repositories.


The `preimage` command-line utility
-----------------------------------

### Synopsis

We first create a file to which we will link:
```shell
$ echo -n 'Hello, World!' > ~/tmp/hello
```

Next, we populate the hash database
```shell
$ preimage scan
```
Then, we may locate the file by its contents:
```shell
$ echo -n 'Hello, World!' | sha256sum
dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f  -
$ preimage find dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f
file:/home/lachlan/tmp/hello
```

If we move the file, its new location will be picked up by the next `preimage
scan` invocation:

```shell
$ mv ~/tmp/hello ~/tmp/world
$ preimage scan
$ preimage find dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f
file:/home/lachlan/tmp/world
```

Using `preimage` with org-mode
------------------------------

Using `org-preimage`, it is possible to link to files or `git` commits by hash.

Following on from the synopsis above, we may obtain a persistent link to the
file containing the text "Hello, World!" by writing

```org
[[preimage:dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f]]
```

This link will continue to work if the file is moved within the search
directory, from the time of the next `preimage scan` invocation.
