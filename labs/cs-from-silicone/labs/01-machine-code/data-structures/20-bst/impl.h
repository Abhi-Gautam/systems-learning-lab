#pragma once
class BinarySearchTree {
  public:
    BinarySearchTree();
    ~BinarySearchTree();
    BinarySearchTree(const BinarySearchTree &) = delete;
    BinarySearchTree &operator=(const BinarySearchTree &) = delete;
    bool insert(int);
    bool contains(int) const;
    bool erase(int);
    int min() const;
    int max() const;

  private:
    struct Node {
        int v;
        Node *l;
        Node *r;
        Node(int x) : v(x), l(nullptr), r(nullptr) {}
    };
    Node *root_;
    void destroy(Node *);
};
