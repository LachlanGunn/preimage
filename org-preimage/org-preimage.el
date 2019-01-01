;;; org-preimage.el - Link to files by hash.
(require 'org)
(org-add-link-type "preimage" 'org-preimage-open)

(defcustom preimage-command "preimage"
  "The location of the preimage executable."
  :type 'file)

(defun org-preimage-open (hash)
  "Visit the object with HASH."
  (let (location type type-location)
    ; Get the <type>:<location> path from the database.
    (setq type-location (shell-command-to-string (concat preimage-command " find " hash)))

    ; Find the separator character and get the type/location
    (setq separator-position (string-match ":" type-location))
    (setq type
          (substring type-location
                     0
                     separator-position))
    (setq location
          (substring type-location
                     (+ separator-position 1)
                     (- (length type-location) 1)))

    ; Open the object.
    (if (string-equal type "file")
        ; If it's a file, open it.
        (progn
          (find-file-existing location)))
    (if (string-equal type "git")
        ; If it's a git commit, show it using magit.
        (progn
          (require 'magit)
          (let* ((default-directory location))
            (magit-show-commit hash))))))
