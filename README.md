# uelaur
stands for Uel Evaluation Leaf Automation Utility in Rust

## Discription
this is a tool I created to automate the tedious task of filling the uel evaluation \
paper, it is simply a wrapper around [leafedit](https://github.com/navyleaf/leafedit)
> ### leafedit
> is a command line pdf editor that I wrote due to my interest in the
> pdf format and love for CLI and Rust, I do plan on developing leafedit into a
> tool than can act as the backend for a gui pdf editor

so all that uelaur does is read the data of a csv file and the co-ordinates of each field
from a toml file then run leafedit with the formated arguments
and leafedit does the heavy lifting

## Usage
first create an new empty directory then place uelaur in that directory an run it,
if all goes well you should see \
`new project intialized` \
now a you will see a directory name **csv_files** and a file called **uelaur_config.txt** in \
the directory you ran uelaur in

now place all the csv file in the the **csv_files directory** \(note all file extentions
are unchecked except for pdf files\)

now configure uelaur by editing **uelaur_config.txt**, you can get the co-ordinates from \
from [pdflite.co](https://pdflite.co/simplified-pdf-viewer/index.html) or better yet
use [pdfbox](https://pdfbox.apache.org/download.cgi) which is my preferred choice, \
it does require java to be installed but it's much better and easier to use

finally place the uel pdf in the directory you ran uelaur in then run uelaur again and \
watch the magic

assuming no error occured uelaur will displays `done !!!` then it will exit after a few seconds and now you will find \
two new directories:

- **uel_papers**: which will contain a directory for every csv file present in the **csv_files** \
each directory will bear the same name \(minus the extention\) as a csv file

- **review**: which will contain a pdf for each directory in uel\_papers \
the pdf file is the result of merging all the pdf files of the directory inside
uel\_papers bearing the same name

it will also contain two files:
- **uel_pdf.patched**: *internal and shouldn't be visible to the user*
- **leafedit(.exe)**: *internal and shouldn't be visible to the user*

but I didn't bother hidding them

## Configuration

configuration file is the toml format, \
all numeric values must be unsigned integers

### csv related options
- **name_column**: the column number containing each students name

- **id_column**: the column number containing each students id

- **mark_column**: the column number containing each students mark

- **final_mark_column**: the column number containing the final mark

note: columns start from 1

### uel pdf related
- **name_postion**: an array containing two  numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the postion of the student name in the pdf

- **id_postion**: an array containing two  numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the postion of the student id in the pdf

- **student_field_font_size**: the font size of the student name and id

- **horizontal_marks**: an array of numbers, contains the marks that are listed in the
criteria fields, in other words each mark that is directly above a check mark box

- **horizontal_postions**: an array having the same lenght as the horizontal\_marks array,
each entry is an array containing 2 values, the x and y co-ordinates of the where the
check mark should be placed if the student mark is greater than the mark after the
current mark

- **horizontal_feild_font_size**: the font size of the check mark used in the horizontal
mark boxes

- **first_marker_postion**: an array containing two  numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the postion of the first marker in the pdf

- **second_marker_postion**: an array containing two  numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the postion of the second marker in the pdf

- **asu_mark_postion**: an array containing two  numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the postion of the asu in the pdf

- **uel_mark_postion**: an array containing two  numbers, the first is the x co-ordinate
and the second is the y co-ordinate of the postion of the uel in the pdf

- **grade_field_font_size**: the font size used of the above fields

- **vertical_marks**: an array containing arrays, each contained array contains to numbers
the first is the upper bound for the uel mark of current range, the second is the upper bound for
the asu mark of the current range, current range is range given to each letter grade,
so A+ will always have an upper bound of 100 therefor the firsr entry will always be
[100, 100], also 0 is not an upper bound to any letter grade so the array must never
contain [0, 0]

- **vertical_postions**: an array having the same lenght as the vertical\_marks array,
each entry is an array containing 2 values, the x and y co-ordinates of the where the
check mark should be placed if the student mark is in the range of the corresponding letter
grade

- **vertical_feild_font_size**: the font size of the check mark used in the vertical
mark boxes

note: all co-ordinates should be taken at the bottom left corner of the field
as the y co-ordinate supplied will act as the baseline for each characters

## Error handling
### csv related errors
if the final mark in the second line of the csv file is not present
at the expected column or isn't a valid number (float/int) then the entire
csv file will be skipped, also the final mark is only required in the second line only

if the name or id is empty then that line is skipped

if the student mark is not present or isn't a number (float/int) then 0 will be used instead

if any error occurred, then a file named **CSV_ERRORS.txt** will be created containing the file
name of each file that generated an error as well as the line number of where the error
occurred and the reason for the error

### configuration related
on the first time an error occurs error in the config file then the **uelaur_config.txt** will
renamed to **config\_backup.txt** and a **uelaur_config.txt** will be created after striping all the comments from the previous file while
preserve the valid options and also adding more information about each error that occurred

if an error occurs for the second time or later then **uelaur_config.txt** will be overwritten
directly, the reason for this is because the **uelaur_config.txt** that is
generated the first time uelaur is run contains useful comments but
**uelaur_config.txt** that is created after first error only contains error messages

#### config error types
- missing: the following keys where not present in the file
- un-expected: the following keys are not part of the config
- misspelled: the following keys are missing and un-expected keys where found and the
edit distance between the two keys is less than 4 operations
- repeated: the following keys where found multiple time even if all occurrences have
the same value
- in-correct: the key is valid but the value associated with it is invalid,
refer to the **config_backup.txt** as it might help

### Other
all other errors are printed to standard out

## Limitations
windows cmd has a feature called QuickEdit which will block all print operations if
the user tries to select text, and will unblock them once any key is pressed and I couldn't
find a way to disable this in code so if the user ever clicks on the console they need to press
a key for messages to be displayed again, \
note that all print operations are called from a separate thread than the main thread so uelaur
could exit without displaying error messages and warnigs


also I don't know how to sign binarys so if you do download uelaur from the here, your browser will block it
or warn you that it's not a commonly download executable, after that if you try to run
it windows will block it with a blue screen because it contains a "mark of the web" and
you will need to right click the executable and goto properties and check the box next to
unblock


the mac version is untested and will most likely fail as the leafedit bundled with uelaur is
unsigned, because I don't own a mac or an apple developer account \
if someone can compile it and sign it I would thankful

## final word
I don't plan on maintaining this project so that's why the variables and function
names aren't discriptive, and the order of function defitions is random \
but if someone is willing to maintain uelaur then I will consider rewriting it in a cleaner way

\
\
\
\
also if it's not asking too much I'd like a few bonus marks as compensation for my I hard work \
*I really need the extra marks*
