import React from 'react';
import styled from '@emotion/styled';
import pt from 'prop-types';
import { arc } from 'd3-shape';

const Button = styled.div`
  width: 3em;
  height: 3em;
  display: flex;

  justify-content: center;
  align-items: center;

  background: rgba(255, 255, 255, 0.95);
  border-radius: 50%;
  color: var(--ij-bg);
`;
const MenuBox = styled.div`
  position: absolute;
  top: 50%;
  left: 50%;
  width: 20em;
  height: 20em;
  transform: translate(-50%, -50%);
`;
const Menu = styled.svg`
  width: 100%;
  height: 100%;
`;

const toAbsolute = (coord, scale = 0.7) => coord * scale + 1;

const Label = styled.div`
  position: absolute;
  display: flex;
  width: 7em;
  height: 7em;
  top: ${(props) => toAbsolute(props.y) * 10 + 'em'};
  left: ${(props) => toAbsolute(props.x) * 10 + 'em'};
  transform: translate(-50%, -50%);

  background: rgba(255, 255, 255, 0.8);

  justify-content: center;
  align-items: center;
`;

// window.arc = arc;

function RadialMenu(props) {
  const [open, setOpen] = React.useState(true);

  const children = React.Children.toArray(props.children).filter(Boolean);
  const svgSize = 512;

  const tao = Math.PI * 2;
  const sliceAngle = tao / children.length;
  // Defined such that the first item will be centered
  const baseAngle = (sliceAngle / 2) * -1;

  const items = children.map((element, i) => {
    const startAngle = baseAngle + sliceAngle * i;
    const endAngle = startAngle + sliceAngle;
    const centerAngle = (endAngle + startAngle) / 2;
    const x = Math.cos(centerAngle - Math.PI / 2);
    const y = Math.sin(centerAngle - Math.PI / 2);
    const arcPath = arc()({
      innerRadius: Math.floor(svgSize / 8),
      outerRadius: Math.floor(svgSize / 2),
      startAngle,
      endAngle,
    });

    return {
      x,
      y,
      arcPath,
      element,
      color: `hsl(${(180 / Math.PI) * startAngle}deg, 100%, 50%)`,
    };
  });

  return (
    <>
      <Button onClick={() => setOpen((o) => !o)}>[]</Button>
      {open && (
        <MenuBox>
          <Menu viewBox={`0 0 ${svgSize} ${svgSize}`}>
            <g transform={`translate(${svgSize / 2}, ${svgSize / 2})`}>
              {items.map((c, i) => (
                <path fill={c.color} d={c.arcPath} key={i} />
              ))}
            </g>
          </Menu>
          {items.map((c, i) => (
            <Label key={i} x={c.x} y={c.y} style={{ color: c.color }}>
              {c.element}
            </Label>
          ))}
        </MenuBox>
      )}
    </>
  );
}

RadialMenu.propTypes = {
  children: pt.node.isRequired,
};

export default RadialMenu;
