import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletInvoicesComponent } from './wallet-invoices.component';

describe('WalletInvoicesComponent', () => {
  let component: WalletInvoicesComponent;
  let fixture: ComponentFixture<WalletInvoicesComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletInvoicesComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletInvoicesComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
