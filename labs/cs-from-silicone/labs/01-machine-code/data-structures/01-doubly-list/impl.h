#pragma once
class DoublyList {
  public:
    struct Node {
        int val;
        Node *prev;
        Node *next;
        explicit Node(int v = 0, Node *p = nullptr, Node *n = nullptr) : val(v), prev(p), next(n) {}
    };
    DoublyList();
    ~DoublyList();
    DoublyList(const DoublyList &) = delete;
    DoublyList &operator=(const DoublyList &) = delete;
    bool empty() const;
    int size() const;
    void insert_between(Node *, Node *, Node *);
    void push_front(int);
    void push_back(int);
    bool pop_front();
    bool pop_back();
    int front() const;
    int back() const;
    Node *insert_after(Node *, int);
    void erase(Node *);
    Node *find(int);
    void clear();
    Node *begin() const;
    Node *end() const;

  private:
    Node *dummy_;
    int size_;
};
