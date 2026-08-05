-- Add up migration script here
CREATE TABLE public.ship_transfer_request (
    id serial NOT NULL,
    ship_symbol character varying NOT NULL,
    reserved_fund bigint NOT NULL,
    finished boolean NOT NULL DEFAULT false,
    fleet_id integer NOT NULL,
    assignment_id bigint NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT fk_ship_symbol FOREIGN KEY (ship_symbol) REFERENCES public.ship_info (symbol) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
    CONSTRAINT fk_fleet FOREIGN KEY (fleet_id) REFERENCES public.fleet (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
    CONSTRAINT fk_assignment FOREIGN KEY (assignment_id) REFERENCES public.ship_assignment (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID,
    CONSTRAINT fk_reserved_fund FOREIGN KEY (reserved_fund) REFERENCES public.reserved_funds (id) MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION NOT VALID
);