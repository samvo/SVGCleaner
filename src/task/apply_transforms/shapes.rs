/****************************************************************************
**
** svgcleaner could help you to clean up your SVG files
** from unnecessary data.
** Copyright (C) 2012-2017 Evgeniy Reizner
**
** This program is free software; you can redistribute it and/or modify
** it under the terms of the GNU General Public License as published by
** the Free Software Foundation; either version 2 of the License, or
** (at your option) any later version.
**
** This program is distributed in the hope that it will be useful,
** but WITHOUT ANY WARRANTY; without even the implied warranty of
** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
** GNU General Public License for more details.
**
** You should have received a copy of the GNU General Public License along
** with this program; if not, write to the Free Software Foundation, Inc.,
** 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
**
****************************************************************************/

use task::short::{EId, AId};
use super::utils;

use svgdom::{Document, Node, Attributes};
use svgdom::types::Transform;

pub fn apply_transform_to_shapes(doc: &Document) {
    // If group has transform and contains only valid shapes
    // we can apply the group's transform to children before applying transform to
    // actual shape's coordinates.
    //
    // We use own implementation, because 'task::ungroup_groups' method does not support it.
    let iter = doc.descendants().svg().filter(|n|    n.is_tag_name(EId::G)
                                                  && n.has_attribute(AId::Transform));

    for node in iter {
        if !utils::has_valid_transform(&node) || !utils::is_valid_attrs(&node) {
            continue;
        }

        // check that all children is valid
        let is_valid = node.children().svg().all(|n| {
            let flag = match n.tag_id().unwrap() {
                  EId::Rect
                | EId::Circle
                | EId::Ellipse
                | EId::Line => true,
                _ => false,
            };

               flag
            && utils::has_valid_transform(&n)
            && utils::is_valid_attrs(&n)
            && utils::is_valid_coords(&n)
        });

        if is_valid {
            let ts = utils::get_ts(&node);

            // apply group's transform to children
            for child in node.children().svg() {
                if child.has_attribute(AId::Transform) {
                    // we should multiply transform matrices
                    let mut ts1 = ts;
                    let ts2 = utils::get_ts(&child);
                    ts1.append(&ts2);
                    child.set_attribute(AId::Transform, ts1);
                } else {
                    child.set_attribute(AId::Transform, ts);
                }
            }

            node.remove_attribute(AId::Transform);

            // we do not remove group element here, because
            // it's 'task::ungroup_groups' job
        }
    }

    // apply transform to shapes
    let iter = doc.descendants().svg().filter(|n| n.has_attribute(AId::Transform));
    for node in iter {
        match node.tag_id().unwrap() {
            EId::Rect => process_rect(&node),
            EId::Circle => process_circle(&node),
            EId::Ellipse => process_ellipse(&node),
            EId::Line => process_line(&node),
            _ => {}
        }
    }
}

fn process<F>(node: &Node, func: F)
    where F : Fn(&mut Attributes, &Transform)
{
    if    !utils::has_valid_transform(node)
       || !utils::is_valid_attrs(node)
       || !utils::is_valid_coords(node) {
        return;
    }

    let ts = utils::get_ts(node);

    {
        let mut attrs = node.attributes_mut();
        func(&mut attrs, &ts);
        attrs.remove(AId::Transform);
    }

    if ts.has_scale() {
        // we must update 'stroke-width' if transform had scale part in it
        let (sx, _) = ts.get_scale();
        ::task::utils::recalc_stroke_width(node, sx);
    }
}

fn process_rect(node: &Node) {
    process(node, |mut attrs, ts| {
        utils::transform_coords(&mut attrs, AId::X, AId::Y, ts);

        if ts.has_scale() {
            let (sx, _) = ts.get_scale();

            utils::scale_coord(&mut attrs, AId::Width, &sx);
            utils::scale_coord(&mut attrs, AId::Height, &sx);

            utils::scale_coord(&mut attrs, AId::Rx, &sx);
            utils::scale_coord(&mut attrs, AId::Ry, &sx);
        }
    });
}

fn process_circle(node: &Node) {
    process(node, |mut attrs, ts| {
        utils::transform_coords(&mut attrs, AId::Cx, AId::Cy, ts);

        if ts.has_scale() {
            let (sx, _) = ts.get_scale();
            utils::scale_coord(&mut attrs, AId::R, &sx);
        }
    });
}

fn process_ellipse(node: &Node) {
    process(node, |mut attrs, ts| {
        utils::transform_coords(&mut attrs, AId::Cx, AId::Cy, ts);

        if ts.has_scale() {
            let (sx, _) = ts.get_scale();
            utils::scale_coord(&mut attrs, AId::Rx, &sx);
            utils::scale_coord(&mut attrs, AId::Ry, &sx);
        }
    });
}

fn process_line(node: &Node) {
    process(node, |mut attrs, ts| {
        utils::transform_coords(&mut attrs, AId::X1, AId::Y1, ts);
        utils::transform_coords(&mut attrs, AId::X2, AId::Y2, ts);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use svgdom::{Document, WriteToString};
    use task::utils;

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            #[test]
            fn $name() {
                let doc = Document::from_str($in_text).unwrap();
                utils::resolve_gradient_attributes(&doc).unwrap();
                apply_transform_to_shapes(&doc);
                assert_eq_text!(doc.to_string_with_opt(&write_opt_for_tests!()), $out_text);
            }
        )
    }

    test!(apply_1,
"<svg>
    <rect height='10' width='10' x='10' y='10' transform='translate(10 20)'/>
</svg>",
"<svg>
    <rect height='10' width='10' x='20' y='30'/>
</svg>
");

    test!(apply_2,
"<svg>
    <rect height='10' rx='2' ry='2' width='10' x='10' y='10' transform='translate(10 20) scale(2)'/>
</svg>",
"<svg>
    <rect height='20' rx='4' ry='4' stroke-width='2' width='20' x='30' y='40'/>
</svg>
");

    test!(apply_3,
"<svg>
    <rect height='10' width='10' transform='translate(10 20) scale(2)'/>
</svg>",
"<svg>
    <rect height='20' stroke-width='2' width='20' x='10' y='20'/>
</svg>
");

    test!(apply_4,
"<svg stroke-width='2'>
    <rect height='10' width='10' transform='scale(2)'/>
</svg>",
"<svg stroke-width='2'>
    <rect height='20' stroke-width='4' width='20' x='0' y='0'/>
</svg>
");

    test!(apply_circle_1,
"<svg>
    <circle cx='10' cy='10' r='15' transform='translate(10 20) scale(2)'/>
</svg>",
"<svg>
    <circle cx='30' cy='40' r='30' stroke-width='2'/>
</svg>
");

    test!(apply_ellipse_1,
"<svg>
    <ellipse cx='10' cy='10' rx='15' ry='15' transform='translate(10 20) scale(2)'/>
</svg>",
"<svg>
    <ellipse cx='30' cy='40' rx='30' ry='30' stroke-width='2'/>
</svg>
");

    test!(apply_line_1,
"<svg>
    <line x1='10' x2='10' y1='15' y2='15' transform='translate(10 20) scale(2)'/>
</svg>",
"<svg>
    <line stroke-width='2' x1='30' x2='30' y1='50' y2='50'/>
</svg>
");

    test!(apply_g_1,
"<svg>
    <g transform='translate(10 20) scale(2)'>
        <rect height='10' width='10' x='10' y='10' transform='scale(2)'/>
        <rect height='10' width='10' x='10' y='10'/>
        <rect height='10' width='10' x='10' y='10'/>
    </g>
</svg>",
"<svg>
    <g>
        <rect height='40' stroke-width='4' width='40' x='50' y='60'/>
        <rect height='20' stroke-width='2' width='20' x='30' y='40'/>
        <rect height='20' stroke-width='2' width='20' x='30' y='40'/>
    </g>
</svg>
");

    // ignore shapes with invalid coordinates units
    test_eq!(keep_1,
"<svg>
    <rect height='10' transform='scale(2)' width='10' x='10in' y='10'/>
</svg>
"
);

    // ignore groups processing with invalid transform types
    // and attributes
    test_eq!(keep_2,
"<svg>
    <g transform='scale(2 3)'>
        <rect height='10' width='10' x='10' y='10'/>
    </g>
    <mask id='m'/>
    <g mask='url(#m)' transform='scale(2)'>
        <rect height='10' width='10' x='10' y='10'/>
    </g>
</svg>
"
);

}
