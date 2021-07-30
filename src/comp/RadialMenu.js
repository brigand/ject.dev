import React from 'react';
import styled from '@emotion/styled';
import pt from 'prop-types';
import { arc } from 'd3-shape';

const MenuBox = styled.div`
  position: absolute;
  top: 50%;
  left: 50%;
  width: 20em;
  height: 20em;
  transform: translate(-50%, -50%);

  animation: radial-menu-scale-in 0.3s normal forwards ease-in-out;

  @keyframes radial-menu-scale-in {
    0% {
      transform: translate(-50%, -50%) scale(0);
    }

    100% {
      transform: translate(-50%, -50%) scale(1);
    }
  }
`;
const Menu = styled.svg`
  width: 100%;
  height: 100%;
`;

const toAbsolute = (coord, scale = 0.6) => coord * scale + 1;

const Label = styled.div`
  position: absolute;
  display: flex;
  width: 7em;
  height: 7em;
  top: ${(props) => toAbsolute(props.y) * 10 + 'em'};
  left: ${(props) => toAbsolute(props.x) * 10 + 'em'};
  transform: translate(-50%, -50%);

  justify-content: center;
  align-items: center;

  // background: rgba(255, 255, 255, 0.8);

  pointer-events: none;
`;

const Button = styled.div`
  width: 4em;
  height: 4em;
  display: flex;

  justify-content: center;
  align-items: center;

  background: rgba(255, 255, 255, 0.95);
  border-radius: 50%;
  color: var(--ij-bg);
  overflow: hidden;

  transition: transform 0.4s ease-in;
  transform: ${(props) =>
    props.open ? `rotateY(180deg) rotate(90deg)` : `rotate(0)`};
`;
Button.propTypes = {
  open: pt.bool,
};

const LogoSvg = styled.svg`
  color: var(--ij-bg);
  width: 4em;
  height: 4em;

  rect,
  path,
  polygon {
    fill: currentColor;
  }
`;

const Logo = (props) => (
  <LogoSvg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" {...props}>
    <circle cx="256" cy="256" r="254" fill="#fff" />
    <path
      d="M120.9,118.6l5.9,16a17.6,17.6,0,0,0,24.4,9.6l42.9-21.3a17.6,17.6,0,0,0,8.7-21.9l-7-18.9A17.6,17.6,0,0,0,170.4,73L128.6,97.2A17.7,17.7,0,0,0,120.9,118.6Z"
      fill="#23262e"
    />
    <polygon
      points="268.5 74.5 305 359 340 358 351.5 68.5 268.5 74.5"
      fill="#23262e"
    />
    <polygon
      points="286.5 379.5 283.5 438.5 382.5 431.5 371.5 373.5 286.5 379.5"
      fill="#23262e"
    />
    <path
      d="M167.2,152.6a17.5,17.5,0,0,0-3,26.7C189.4,205.6,233.9,282,191,371c-40,83-75-28-82.7-60.4a17.5,17.5,0,0,0-20.4-13l-9.5,1.9a17.5,17.5,0,0,0-12.9,23.7C82.4,365.3,126,458.5,174,432.5c61.5-33.2,79-141.4,75.1-180.2-2.7-27.6-20-75.8-38.2-101.7C200.3,135.4,181.2,143.4,167.2,152.6Z"
      fill="#23262e"
    />
  </LogoSvg>
);

const Arc = styled.path`
  fill: var(--ij-bg-alt);
  cursor: pointer;

  &:hover {
    fill: var(--ij-bg-alt-2);
  }
`;

function RadialMenu(props) {
  const status = React.useRef(null);
  const [open, setOpen] = React.useState(false);

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
      {open && (
        <MenuBox>
          <Menu viewBox={`0 0 ${svgSize} ${svgSize}`}>
            <g transform={`translate(${svgSize / 2}, ${svgSize / 2})`}>
              {items.map((c, i) => (
                <Arc
                  d={c.arcPath}
                  key={i}
                  onMouseDown={(event) => {
                    event.preventDefault();
                    status.current = `menu-${i}`;
                  }}
                  onMouseLeave={() => {
                    if (status.current === `menu-${i}`) {
                      status.current = null;
                    }
                  }}
                  onMouseUp={() => {
                    if (status.current === `menu-${i}`) {
                      status.current = null;
                      children[i].props?.onClick();
                      setOpen(false);
                    }
                  }}
                />
              ))}
            </g>
          </Menu>
          {items.map((c, i) => (
            <Label key={i} x={c.x} y={c.y}>
              {c.element}
            </Label>
          ))}
        </MenuBox>
      )}
      <Button
        open={open}
        onMouseDown={(event) => {
          event.preventDefault();
          status.current = 'logo';
        }}
        onMouseLeave={() => {
          if (status.current === 'logo') {
            status.current = null;
          }
        }}
        onMouseUp={() => {
          if (status.current === 'logo') {
            status.current = null;
            setOpen((c) => !c);
          }
        }}
      >
        <Logo />
      </Button>
    </>
  );
}

RadialMenu.propTypes = {
  children: pt.node.isRequired,
};

export default RadialMenu;
