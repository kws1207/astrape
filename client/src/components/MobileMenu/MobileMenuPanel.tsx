"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useWindowSize } from "usehooks-ts";

import Icon from "@/components/Icons";
import useStore from "@/stores/store";
import { MODAL_NAMES } from "@/utils/constant";

import Modal from "../Modal/Modal";

import styles from "./styles.module.scss";

export default function MobileMenuPanel() {
  const pathname = usePathname();
  const currentModal = useStore((state) => state.currentModal);
  const closeModal = useStore((state) => state.closeModal);
  const { width = 0 } = useWindowSize();
  const isTablet = width < 1024;

  return (
    <Modal
      width="450px"
      isOpen={currentModal === MODAL_NAMES.MOBILE_MENU && isTablet}
      isPositioned={true}
      isDrawer={false}
      topPosition="60px"
      rightPosition="24px"
      onClose={closeModal}
    >
      <div className={styles.mobileMenu}>
        <Link
          href={"/"}
          className={`${styles.mobileMenu__link} ${pathname === "/" ? styles.activeLink : ""}`}
          onClick={() => {
            closeModal();
          }}
        >
          <span className={styles.mobileMenu__link__text}>Home</span>
        </Link>
        <Link
          href={"/mint"}
          className={`${styles.mobileMenu__link} ${pathname === "/mint" ? styles.activeLink : ""}`}
          onClick={() => {
            closeModal();
          }}
        >
          <Icon name="Provide" />
          <span className={styles.mobileMenu__link__text}>Mint</span>
        </Link>
        <Link
          href="/deposit"
          className={`${styles.mobileMenu__link} ${pathname === "/deposit" ? styles.activeLink : ""} relative`}
          onClick={closeModal}
        >
          <Icon name="Claim" />
          <span className={styles.mobileMenu__link__text}>Deposit</span>
        </Link>
        <Link
          href="/dashboard"
          className={`${styles.mobileMenu__link} ${pathname === "/dashboard" ? styles.activeLink : ""}`}
          onClick={closeModal}
        >
          <Icon name="Network" />
          <span className={styles.mobileMenu__link__text}>Dashboard</span>
        </Link>
      </div>
    </Modal>
  );
}
