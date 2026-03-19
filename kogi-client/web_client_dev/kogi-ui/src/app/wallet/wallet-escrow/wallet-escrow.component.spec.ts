import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletEscrowComponent } from './wallet-escrow.component';

describe('WalletEscrowComponent', () => {
  let component: WalletEscrowComponent;
  let fixture: ComponentFixture<WalletEscrowComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletEscrowComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletEscrowComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
