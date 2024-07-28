CREATE TABLE issue_statuses (
    id INTEGER NOT NULL
        CONSTRAINT pk_issue_statuses
        PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
        CONSTRAINT un_issue_statuses_name
        UNIQUE
);

CREATE TABLE issues (
    id INTEGER NOT NULL
        CONSTRAINT pk_issues
        PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status INTEGER NOT NULL
        CONSTRAINT fk_issues_status
        REFERENCES issue_statuses (id)
        ON UPDATE RESTRICT
        ON DElETE RESTRICT,
    parent INTEGER DEFAULT NULL
        CONSTRAINT fk_issues_parent
        REFERENCES issues (id)
        ON UPDATE CASCADE
        ON DELETE SET NULL
);

CREATE TABLE issue_blockings (
    id INTEGER NOT NULL
        CONSTRAINT pk_issue_blockings
        PRIMARY KEY AUTOINCREMENT,
    blocker INTEGER NOT NULL
        CONSTRAINT fk_issue_blocker
        REFERENCES issues (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    blocked INTEGER NOT NULL
        CONSTRAINT fk_issue_blocked
        REFERENCES issues (id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);
