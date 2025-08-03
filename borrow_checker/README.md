# Rust::Владение и заимствоваание

Концепция "владения и заимствования" используется для своевременного **автоматического** освобождения ранее выделенных ресурсов без использования *Garbage collector'а*.

Сопровождается контролем доступа к ресурсу, чтобы избежать обращения к нему уже после его освобождения.

Таким ресурсом обычно является область памяти, но может быть и файловый дескриптор и ему подобные.

*Garbage collector'ом* называется часть *runtime'а*, ответственная за "сборку мусора" во время выполнения программы.

*Runtime* - это служебная логика, обеспечивающий выполнение кода программы.

## Решаемые проблемы 

- ошибка [Segmentation fault](https://en.wikipedia.org/wiki/Segmentation_fault). Возникает при обращении к ранее выделенному, но уже освобожденному участку памяти

- "утечка памяти" (["memory leak"](https://en.wikipedia.org/wiki/Memory_leak))". Возникает, когда не происходит освобождение ранее выделенного участка памяти

## Ретроспектива 

На заре программирования (язык С) освобождение ресурсов в программе производилось раработчиком "вручную" (["manual memory management"](https://en.wikipedia.org/wiki/Manual_memory_management)).

Столкнувшись со сложностью этой задачи ([многочисленными ошибками](https://en.wikipedia.org/wiki/Memory_safety), связанными с этим, особенно в сложных проектах с участием большого количества разработчиков), в runtime'ах языков программирования (Python, Java, JavaScript, Go) появился [Garbage collector](https://ru.wikipedia.org/wiki/%D0%A1%D0%B1%D0%BE%D1%80%D0%BA%D0%B0_%D0%BC%D1%83%D1%81%D0%BE%D1%80%D0%B0), автоматически освобождающий уже неиспользуемые ресурсы.

В результате решения задачи автоматического освобождения ранее выделенных ресурсов были добавлены [новые проблемы](https://ru.wikipedia.org/wiki/%D0%A1%D0%B1%D0%BE%D1%80%D0%BA%D0%B0_%D0%BC%D1%83%D1%81%D0%BE%D1%80%D0%B0#%D0%9F%D1%80%D0%BE%D0%B1%D0%BB%D0%B5%D0%BC%D1%8B_%D0%B8%D1%81%D0%BF%D0%BE%D0%BB%D1%8C%D0%B7%D0%BE%D0%B2%D0%B0%D0%BD%D0%B8%D1%8F):
- ошибка *Garbage Collector Fault*, когда работа программы прекращается из-за сбоя в сборщике мусора, являющегося частью *runtime'а*
- непредсказуемые и неконтролируемые freezе"ы: на время сборки мусора *Garbage Collector* приостанавливает выполнение программы, возобнвляя его после окончания процедуры
- снижение эффективности использования ресурсов (памяти), поскольку их освобождение происходит не в момент прекращения использования, а в момент непредсказуемого и неконтролируемого следующего цикла работы *Garbage Collector'а*

В стане языков с "ручным" управление ресурсов (язык C++) также предпринимались попытки автоматизации освобождения ресурсов, например, с использованием ["умных указателей"](https://ru.wikipedia.org/wiki/%D0%A3%D0%BC%D0%BD%D1%8B%D0%B9_%D1%83%D0%BA%D0%B0%D0%B7%D0%B0%D1%82%D0%B5%D0%BB%D1%8C) со счётчиком ссылок.

## Концепция

Следующим прорывным решением задачи автоматического освобождения ранее выдыленных ресурсов стала [концепция "владения и заимствования"](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html), которая основывается на следующих простых правилах:

### Правило владения (ownership)

**У каждого ресурса может быть только один владелец (переменная) в один момент времени.**

Владение позволяет как читать, так и изменять (mutate) ресурс. 

При этом владение может передаваться от одного владельца другому. Процедура передачи владения называется "move". 

В момент прекращения владения (в случае выхода переменной-владельца из зоны видимости или после явного заявления о прекращении владения путем вызова функции `drop`) ресурс немедленно освобождается.

Если у ресурса нет владельца (например, после вызова `drop`), то доступ к нему запрещен, поскольку ресурс уже освобожден.

### Правила заимствования (borrow)

1. Остальные (переменные, параметры функции), нуждающиеся в ресурсе, могут получить доступ к нему в режиме заимствования ("borrow"), то есть в виде ссылки на ресурс.

2. В один момент времени может существовать либо несколько "ссылок на чтение" (`&`), либо только одна "изменяемая ссылка" (`&mut`), которая позволяет получить доступ к ресурсу как на чтение, так и на запись (изменение).

### Borrow Checker

*Borrow Checker* - это часть логики **компилятора**, 
- проверяющая следование разработчиком "правилам владения и заимствования", 
- позволяющая компилятору сформировать стратегию освобождения ресурсов еще **на этапе сборки** приложения, а не во время его выполнения, как в случае использования *Garbage Collector'а*. 

Отказ от последнего позволяет устранить и вносимые им проблемы, поскольку "деталь, которой нет, не может сломаться"

Разумеется, изящно решая одну проблему, Borrow Checker создает другую - "крутую кривую обучения" (steep learning curve), которая требует от разработчика [перестройки его навыков](https://corrode.dev/blog/flattening-rusts-learning-curve/) работы с ресурсами (с памятью) при переходе с других языков программирования. 

Нередко, в поытке отрицания "правил владения и заимствования" разработчик вступает в [борьбу с Borrow Checker'ом](https://docs.rs/you-can/latest/you_can/attr.turn_off_the_borrow_checker.html),. Однако единственным правльным решением является принятие ["правил игры"](https://rustc-dev-guide.rust-lang.org/borrow_check.html)  

#### Примеры на С++

##### test_access_after_delete 

```
void test_access_after_delete() {
    int32_t* ptr = new int32_t(10);
    delete ptr;
    *ptr = 12;
}

```
1. Выделяем память в heap ("куче").
2. Освобождаем память.
3. Пытаемся изменить содержимое памяти по указателю.

В результате получаем ошибку, которую крайне сложно найти, потому что:
- компилятор не выдает никаких ошибок или предупреждений,
- в 99.99% случаев этот код успешно выполняется,
- только в определённых, крайне редких случаях программа будет себя неадекватно вести либо падать.

##### test_access_after_add (C++)

С++ контейнеры уменьшают количество таких ошибок ("умные указатели"), но не исключают их полностью:
```
std::vector<int32_t> test_access_after_add() {
    std::vector<int32_t> vec {1, 2, 3};
    // vec.reserve(20);
    int32_t& x2 = vec[];
    vec.push_back(5);
    x2 = 10;
    return vec;
}
```

- Строка `vec.push_back(5);` добавляет элемент и в некоторых версиях STL может привести к тому, что массив расширится за счёти перевыделение памяти 
- Строка `x2 = 10;` запишет новое значение в старую область памяти. 
- Никаких ошибок ни во время компиляции, ни во время исполнения не будет. 
- Будет логическая ошибка: можно проверить элемент `vec[2]` после вызова функции и убедиться, то `vec[2] == 3`, а не `10`. 
- Впрочем, если раскомментировать строку `vec.reserve(20);`, то внутри `vec[2]` будет именно значение `10`. И только в крайне редких случаях будет падение, либо неадекватное поведение программы


#### Примеры на Rust

Вот как комментирует *Borrow Checker* аналогичные попытки на Rust:

##### test_access_after_delete 

```
fn test_access_after_delete() {
    let mut ptr = Box::new(10);
    drop(ptr);
    *ptr = 12;
}
```
Компиляция [вышеуказанного кода](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=84ed084c0b482e53e134f0446ad1e5f2) приводит к следующей диагностике:

```
error[E0382]: use of moved value: `ptr`
 --> src/lib.rs:4:5
  |
2 |     let mut ptr = Box::new(10);
  |         ------- move occurs because `ptr` has type `Box<i32>`, which does not implement the `Copy` trait
3 |     drop(ptr);
  |          --- value moved here
4 |     *ptr = 12;
  |     ^^^^^^^^^ value used here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     drop(ptr.clone());
  |             ++++++++

For more information about this error, try `rustc --explain E0382`.
```

Строки `drop(ptr);` и `*ptr = 12;` конфликтуют за доступ к переменной `ptr`. Поэтому удаление одной из этих строк убирает конфликт. Обычно в программе на Rust не требуется явного вызова drop. Т.к. ресурсы, которыми владеют переменные, удаляются при выходе переменной за пределы области видимости. 

Если посмотреть на определение функции `drop`, то это будет *пустая* функция, которая ничего не делает: `pub fn drop(_x: T) {}`. 

Единственное назначение этой функция - получение владения (ownership) значением переменной. При выходе из функции `drop` ресурс, которым владела переменная, автоматически удаляется.


##### test_access_after_add 

Аналогично со std::vector:
```
fn test_access_after_add() -> Vec<i32> {
    let mut vec = vec![1, 2, 3];
    let x2: &mut i32 = &mut vec[2];
    vec.push(5);
    *x2 = 10;
    return vec;
}
```

Компиляция [вышеуказанного кода](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=eaa7440d3e5d0387c0525cf101860991) приводит к следующей диагностике:

```
error[E0499]: cannot borrow `vec` as mutable more than once at a time
 --> src/lib.rs:4:5
  |
3 |     let x2: &mut i32 = &mut vec[2];
  |                             --- first mutable borrow occurs here
4 |     vec.push(5);
  |     ^^^ second mutable borrow occurs here
5 |     *x2 = 10;
  |     -------- first borrow later used here

For more information about this error, try `rustc --explain E0499`.
```

Здесь мы видим нарушения второго правила заимствования: *В один момент времени может существовать либо несколько "ссылок на чтение" (`&`), либо только одна "ссылка на запись" (`&mut`)*. 

В строке номер 3 мы получаем первую "ссылку на запись" (`&mut`). 

Для выполнения следующей строки нам требуется вторая "ссылка на запись" (`&mut`).

##### multithreaded_merge_sort_internal

Пример проверки в многопоточном случае:
```
fn multithreaded_merge_sort_internal(arr: &mut [i32], tmp: &mut [i32], div_count: i32) {
    let div_point = arr.len() / 2;
    let (left_tmp, right_tmp) = tmp.split_at_mut(div_point);
    let (left_arr, right_arr) = arr.split_at_mut(div_point);
    
    crossbeam::scope(|s| {
        let thread_l = s.spawn(|_| { 
            if div_count > 0 {
                multithreaded_merge_sort_internal(left_arr, left_tmp, div_count - 1);
            } else {
                stable_merge_sort_internal(left_arr, left_tmp);
            }
        });

        let thread_r = s.spawn(|_| { 
            if div_count > 0 {
                multithreaded_merge_sort_internal(left_arr, right_tmp, div_count - 1);
            } else {
                stable_merge_sort_internal(right_arr, right_tmp);
            }
        });

        thread_l.join().unwrap();
        thread_r.join().unwrap();
    }).unwrap();

    stable_merge(arr, tmp, div_point)
}
```

Компиляция [вышеуказанного кода](https://play.rust-lang.org/?version=stable&mode=debug&edition=2024&gist=75f827aecaf287b3bdca7a9a9fb1dc81) приводит к следующей диагностике:

```
error[E0499]: cannot borrow `*left_arr` as mutable more than once at a time
  --> src/lib.rs:15:32
   |
7  |         let thread_l = s.spawn(|_| { 
   |                                --- first mutable borrow occurs here
8  |             if div_count > 0 {
9  |                 multithreaded_merge_sort_internal(left_arr, left_tmp, div_count - 1);
   |                                                   -------- first borrow occurs due to use of `*left_arr` in closure
...
15 |         let thread_r = s.spawn(|_| { 
   |                                ^^^ second mutable borrow occurs here
16 |             if div_count > 0 {
17 |                 multithreaded_merge_sort_internal(left_arr, right_tmp, div_count - 1);
   |                                                   -------- second borrow occurs due to use of `*left_arr` in closure
...
23 |         thread_l.join().unwrap();
   |         -------- first borrow later used here

For more information about this error, try `rustc --explain E0499`.
```


Это основной фрагмент алгоритма для многопоточной сортировки слиянием. 

На вход приходят два массива arr - сортируемый массив, и tmp - массив для хранения временных данных. 

В строчке `let (left_arr, right_arr) = arr.split_at_mut(div_point);` массив делится на два непересекающихся изменяемых slice'а, потому мы их можем сортировать независимо. 

Создаём два потока `thread_l` и `thread_r`. 

Передаем `left_arr`, `right_arr` в эти потоки. 

Ошибка закралась в строчке `multithread_merge_sort_internal(left_arr, right_tmp, div_count-1);`. 

В ней надо было написать `right_arr`. Но здесь намеренно допущена ошибка и указан `left_arr`. 

Borrow checker выявляет факт попытки передачи одной и той же "изменяемой ссылки" (`&mut`) в два разных потока.

## Связанные темы

- Концепция "владения и заимствования" находит своё отражение при [работе с итераторами](https://habr.com/ru/articles/499108/)

- Если по каким-то причинам не удается воспользоваться borrow checker'ом компилятора, то в Rust есть структуры данных, такие как [std::cell::RefCell](https://doc.rust-lang.org/std/cell/struct.RefCell.html) (для использования в однопоточном режиме) и [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html) (для использования в многопоточном режиме. При этом для последнего есть вариант ([tokio::sync::RwLock](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html)) для использования в комбинированном режиме "асинхронной многопоточности", поддерживаемым фреймворком [tokio](https://docs.rs/tokio/latest/tokio/)

- трейт [std::clone::Clone](https://doc.rust-lang.org/std/clone/trait.Clone.html), 

- маркер-трейт [std::marker::Copy](https://doc.rust-lang.org/std/marker/trait.Copy.html)

- структура данных [std::borrow::Cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html) 

- концепция "времени жизни" ([lifetime](https://doc.rust-lang.org/rust-by-example/scope/lifetime.html)) и [распространенные заблуждения, связанные с ней](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md)

- типы "замыканий" (["closure types"](https://doc.rust-lang.org/reference/types/closure.html))

- ["slice"](https://doc.rust-lang.org/book/ch04-03-slices.html)

## Задания для проверки усвоенного материала

[Quiz](https://danlevy.net/quiz-is-your-memory-rusty/)


