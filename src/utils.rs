use minidom::Element;

pub fn find_elems(root: &Element, tag: String) -> Vec<&Element> {
    let mut elems: Vec<&Element> = Vec::new();

    for el in root.children() {
        if el.name() == tag {
            elems.push(el);
        } else {
            let ret = find_elems(el, tag.clone());
            elems.extend(ret);
        }
    }
    elems
}

pub fn find_elem(root: &Element, tag: String) -> Option<&Element> {
    if root.name() == tag {
        return Some(root);
    }

    for el in root.children() {
        if el.name() == tag {
            return Some(el);
        } else {
            let ret = find_elem(el, tag.clone());
            if ret.is_some() {
                return ret;
            }
        }
    }
    None
}
