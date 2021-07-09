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
  width: 3.1em;
  height: 3.1em;

  rect,
  path,
  polygon {
    fill: currentColor;
  }
`;

const Logo = (props) => (
  <LogoSvg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" {...props}>
    <rect x="123.5" y="0.5" width="384" height="98" rx="28" />
    <rect x="7" width="100" height="100" rx="5.7" />
    <path
      d="M511,176.1c.3,2.7.6,24.9,1.5,63.8s-62.1,48.5-67.6,49.9l-1.4.2H44c-22.9,0-41.5-17.6-41.5-39.2v-8.9c0-18.8,16.2-34.1,36.1-34.1l369.6-1c7.9,0,14.3-6.1,14.3-13.5h0a9,9,0,0,0-6.9-8.7l-1.2-.2-54.5-.8c-19.4-.4-37-11.3-44.5-28.2-4.4-10-4.8-20,6.8-25l.7-.3c6.2-1.8,123.7-26.7,183.5,17.5a5.2,5.2,0,0,1,2.1,4.1"
      transform="translate(0)"
    />
    <polygon points="0 323.5 0 512.1 345 443.5 345 364.8 0 323.5" />
    <polygon points="367.5 368.5 367.5 444 503.3 444 500.5 368.5 367.5 368.5" />
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
