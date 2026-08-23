#pragma once
class OpenAddressMap{public:OpenAddressMap();~OpenAddressMap();OpenAddressMap(const OpenAddressMap&)=delete;OpenAddressMap&operator=(const OpenAddressMap&)=delete;void put(int,int);int get(int);bool remove(int);private:struct Slot{int k,v;unsigned char state;};Slot*slots_;int cap_;int size_;int hash(int)const;void resize();};
