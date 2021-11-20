use crate::widget::cell2::Entry;

/// Return an expanded list of paths based on the given stack of indices.
///
/// For example: If the stack contains [4, 5, 2] the result would be
/// [[4], [4, 5], [4, 5, 2]].
#[must_use]
pub fn stack_to_path_list(stack: &[usize]) -> Vec<&[usize]> {
    let mut list = Vec::new();

    for i in 0..stack.len() + 1 {
        list.push(&stack[0..i])
    }

    
    list
}

/// Returns the entry specified by the given path.
///
/// # Panics
///
/// Will panic if the entry does not exists or the path is invalid.
#[must_use]
pub fn get_entry<'a, Message, Renderer>(
    entries: &'a Vec<Entry<'a, Message, Renderer>>,
    path: &[usize],
) -> &'a Entry<'a, Message, Renderer> {
    get_entry_internal(entries, &path[..])
}

/// Returns the entry of the section specified by the given path.
///
/// # Panics
///
/// Will panic if the entry does not exists or the path is invalid.
fn get_entry_internal<'a, Message, Renderer>(
    entries: &'a [Entry<'a, Message, Renderer>],
    path: &[usize],
) -> &'a Entry<'a, Message, Renderer> {
    if path.len() == 1 {
        &entries[path[0]]
    } else {
        match &entries[path[0]] {
            Entry::Group(_, entries) => get_entry_internal(entries, &path[1..]),
            Entry::Separator => unreachable!("Illegal to get a seperator!"),
            _ => &entries[path[0]]
        }
    }
}


/// Returns a list of all entries in the given section grouped together by its
/// `Entry::Group` based on the given path.
///
/// # Panics
///
/// Will panic if the given path is invalid.
#[must_use]
pub fn get_entry_list<'a, Message, Renderer>(
    all_entries: &'a Vec<Entry<'a, Message, Renderer>>,
    path: &[usize],
) -> Vec<&'a Vec<Entry<'a, Message, Renderer>>> {
    let mut entries = Vec::new();
    entries.push(all_entries);
    get_entry_list_internal(&all_entries, &path[..], &mut entries);

    entries
}

/// Returns a list of all entries in the given section grouped together by its
/// `Entry::Group` based on the given path.
///
/// # Panics
///
/// Will panic if the given path is invalid.
fn get_entry_list_internal<'a, Message, Renderer>(
    entries: &'a [Entry<'a, Message, Renderer>],
    path: &[usize],
    collection: &mut Vec<&'a Vec<Entry<'a, Message, Renderer>>>,
) {
    if path.is_empty() {
        return;
    }

    match &entries[path[0]] {
        Entry::Group(_, entries) => {
            collection.push(entries);
            get_entry_list_internal(entries, &path[1..], collection);
        }
        _ => unreachable!("Entry is not a group"),
    }
}

