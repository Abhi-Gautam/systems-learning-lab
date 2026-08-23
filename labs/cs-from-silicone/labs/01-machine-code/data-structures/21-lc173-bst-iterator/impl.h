#pragma once
struct TreeNode{int val;TreeNode*left;TreeNode*right;explicit TreeNode(int x):val(x),left(nullptr),right(nullptr){}};
class BSTIterator{public:explicit BSTIterator(TreeNode*root);~BSTIterator();BSTIterator(const BSTIterator&)=delete;BSTIterator&operator=(const BSTIterator&)=delete;int next();bool hasNext()const;private:struct Link{TreeNode*n;Link*next;Link(TreeNode*x,Link*y=nullptr):n(x),next(y){}};Link*stack_;void push_left(TreeNode*);};
